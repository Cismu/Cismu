use crate::rating::{AvgRating, Rating};
use lofty::{
    id3::v2::{Frame, FrameId, Id3v2Tag},
    tag::{ItemKey, Tag, TagType},
};

fn rating_str_to_avg(raw: &str) -> AvgRating {
    let txt = raw.trim().replace(',', ".");
    let Ok(mut v) = txt.parse::<f32>() else {
        return AvgRating::Unrated;
    };

    // convertir a porcentaje
    if txt.contains('.') {
        if v <= 1.0 {
            v *= 100.0;
        } else if v <= 5.0 {
            v *= 20.0;
        }
    }

    v = v.clamp(0.0, 100.0);

    if v == 0.0 {
        AvgRating::Unrated
    } else {
        Rating::new(v / 20.0).into()
    }
}

fn popm_byte_to_avg(b: u8) -> AvgRating {
    if b == 0 {
        AvgRating::Unrated
    } else {
        let pct = (b as f32) * (100.0 / 255.0);
        Rating::new(pct / 20.0).into()
    }
}

fn find_unknown_bytes<'a>(tag: &'a Tag, key: &str) -> Option<&'a [u8]> {
    tag.items().find_map(|i| match i.key() {
        ItemKey::Unknown(k) if k.eq_ignore_ascii_case(key) => i.value().binary(),
        _ => None,
    })
}
fn find_unknown_text<'a>(tag: &'a Tag, key: &str) -> Option<&'a str> {
    tag.items().find_map(|i| match i.key() {
        ItemKey::Unknown(k) if k.eq_ignore_ascii_case(key) => i.value().text(),
        _ => None,
    })
}

impl From<&Tag> for AvgRating {
    fn from(tag: &Tag) -> Self {
        match tag.tag_type() {
            TagType::Id3v2 => parse_id3v2(tag),
            TagType::VorbisComments | TagType::Ape => tag
                .get(&ItemKey::Popularimeter)
                .and_then(|item| {
                    item.value()
                        .binary()
                        .filter(|bytes| bytes.len() == 4)
                        .and_then(|bytes| bytes.try_into().ok())
                        .map(|array| u32::from_be_bytes(array).to_string())
                        .or_else(|| item.value().text().map(String::from))
                })
                .or_else(|| find_unknown_text(tag, "RATING").map(String::from))
                .map(|rating_str| rating_str_to_avg(&rating_str))
                .unwrap_or(AvgRating::Unrated),
            _ => AvgRating::Unrated,
        }
    }
}

fn parse_id3v2(tag: &Tag) -> AvgRating {
    let id3 = Id3v2Tag::from(tag.clone());
    if let Some(Frame::Popularimeter(p)) = id3.get(&FrameId::Valid("POPM".into())) {
        return popm_byte_to_avg(p.rating);
    }
    if let Some(bytes) = find_unknown_bytes(tag, "POPM") {
        if let Some(pos) = bytes.iter().position(|&b| b == 0) {
            if let Some(&rating) = bytes.get(pos + 1) {
                return popm_byte_to_avg(rating);
            }
        }
    }
    AvgRating::Unrated
}

#[cfg(test)]
mod tests {
    use super::*;
    use lofty::tag::{ItemKey, ItemValue, Tag, TagItem, TagType};

    #[test]
    fn test_rating_str_to_avg() {
        assert_eq!(rating_str_to_avg("0"), AvgRating::Unrated);
        assert_eq!(rating_str_to_avg("0.73"), Rating::new(3.65).into()); // 73 %
        assert_eq!(rating_str_to_avg("3.4"), Rating::new(3.4).into()); // 3.4 ★
        assert_eq!(rating_str_to_avg("15"), Rating::new(0.75).into()); // 15 %
        assert_eq!(rating_str_to_avg("50"), Rating::new(2.5).into()); // 50 %
        assert_eq!(rating_str_to_avg("100"), Rating::new(5.0).into()); // 100 %
        assert_eq!(rating_str_to_avg("foo"), AvgRating::Unrated);
    }

    #[test]
    fn test_ape_rating() {
        let mut tag = Tag::new(TagType::Ape);
        tag.insert_unchecked(TagItem::new(
            ItemKey::Unknown("RATING".into()),
            ItemValue::Text("20".into()),
        ));
        assert_eq!(AvgRating::from(&tag), Rating::new(1.0).into()); // 20 % → 1.0 ★
    }

    #[test]
    fn test_vorbis_rating() {
        let mut tag = Tag::new(TagType::VorbisComments);
        tag.insert_unchecked(TagItem::new(
            ItemKey::Unknown("RATING".into()),
            ItemValue::Text("80".into()),
        ));
        assert_eq!(AvgRating::from(&tag), Rating::new(4.0).into()); // 80 %
    }

    #[test]
    fn test_id3v2_popm_native() {
        use lofty::id3::v2::PopularimeterFrame;
        let popm = PopularimeterFrame::new("foo".into(), 200, 0);
        let mut id3 = Id3v2Tag::default();
        id3.insert(Frame::Popularimeter(popm));
        let tag = Tag::from(id3);
        let expected = Rating::new((200.0 * 100.0 / 255.0) / 20.0).into(); // ≈ 3.92 ★
        assert_eq!(AvgRating::from(&tag), expected);
    }
}
