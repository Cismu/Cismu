let window_attributes = WindowAttributes::default()
.with_title("OpenGL window")
.with_transparent(true);

// let template = ConfigTemplateBuilder::new().with_alpha_size(8);
// let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

let window = event_loop.create_window(window_attributes).unwrap();
let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

// Since glutin by default tries to create OpenGL core context, which may not be
// present we should try gles.
let fallback_context_attributes = ContextAttributesBuilder::new()
.with_context_api(ContextApi::Gles(None))
.build(raw_window_handle);

// There are also some old devices that support neither modern OpenGL nor GLES.
// To support these we can try and create a 2.1 context.
let legacy_context_attributes = ContextAttributesBuilder::new()
.with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
.build(raw_window_handle);

// Reuse the uncurrented context from a suspended() call if it exists, otherwise
// this is the first time resumed() is called, where the context still
// has to be created.
// let gl_display = gl_config.display();

let window_id = window.id();
let window_status = WindowState {
window: Arc::new(window),
};

self.windows.insert(window_id, window_status);