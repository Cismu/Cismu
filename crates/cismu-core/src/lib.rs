mod discography;

use discography::artist::Artist;
use discography::genre_styles::Genre;

mod tests {
    use super::*;

    #[test]
    fn test() {
        let bio = Some(indoc::indoc!("
            Hatsune Miku is a Japanese virtual singer who single-handedly made the concept of virtual singers popular around the world.
            Her voicebank (based on Saki Fujita's voice) is distributed by Crypton Future Media (code name CV01) since August 31, 2007 for the proprietary Vocaloid software developed by Yamaha. Her anthropomorphous appearance was designed by Japanese illustrator Kei Garou: she is depicted as a 16-year-old idol with long turquoise twintails. Over the years, her appearance superseded her virtual aspect and she now regularly makes live appearances through the use of animated projection technologies.
            Miku Append is an optional upgrade for the base voicebank that should be credited as an ANV.
        ").to_string());

        let the_best_artist = Artist {
            id: 1,
            name: "初音ミク".to_string(),
            variations: vec!["Miku Hatsune".to_string()],
            bio,
            sites: vec![
                "https://ec.crypton.co.jp/pages/prod/virtualsinger/cv01".to_string(),
                "https://www.youtube.com/user/HatsuneMiku".to_string(),
                "https://www.instagram.com/cfm_miku_official/".to_string(),
                "https://www.facebook.com/HatsuneMikuOfficialPage/".to_string(),
                "https://twitter.com/cfm_miku".to_string(),
                "https://piapro.net/intl/en.html".to_string(),
            ],
            genres: vec![Genre::Pop, Genre::Rock, Genre::Electronic],
        };
    }
}
