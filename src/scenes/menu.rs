use scenes::common::*;
use scenes::scene::{Scene, SceneInstance, BaseSwitcher, Switcher};
use scenes::play::Play;
use find_folder;
use conrod::{self, widget, Colorable, Positionable, Widget, Labelable, Sizeable};
use piston_window::*;

widget_ids!(struct Ids { text, button, input, canvas });

pub struct Menu {
    switcher: BaseSwitcher,
    ui: conrod::Ui,
    ids: Ids,
    image_map: conrod::image::Map<G2dTexture>,
    glyph_cache: conrod::text::GlyphCache,
    text_texture_cache: G2dTexture,
    input_text: String
}

impl Menu {
    pub fn new(text_texture_cache: G2dTexture) -> Menu {
        let mut ui = conrod::UiBuilder::new([800., 600.]).build();

        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/Terminus.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;

        Menu {
            switcher: BaseSwitcher::new(None),
            ids: Ids::new(ui.widget_id_generator()),
            ui: ui,
            image_map: conrod::image::Map::<G2dTexture>::new(),
            glyph_cache: conrod::text::GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE),
            text_texture_cache: text_texture_cache,
            input_text: String::from("127.0.0.1:7001")
        }
    }
}

impl Scene for Menu {
    fn handle_event(&mut self, event: Event) {
        let size = (800. as conrod::Scalar,  600. as conrod::Scalar);

        if let Some(e) = conrod::backend::piston::event::convert(event, size.0, size.1) {
            self.ui.handle_event(e);
        }
    }

    fn update(&mut self, _dt: f64) -> GameResult<()> {
        // Set the widgets.
        let ui = &mut self.ui.set_widgets();

        widget::Canvas::new()
            .color(conrod::color::DARK_CHARCOAL)
            .set(self.ids.canvas, ui);

        widget::Text::new("SIDE_RUN")
            .center_justify()
            .middle_of(ui.window)
            .color(conrod::color::WHITE)
            .font_size(32)
            .set(self.ids.text, ui);

        for edit in widget::TextEdit::new(&self.input_text)
            .down_from(self.ids.text, 60.0)
            .set(self.ids.input, ui)
            {
                self.input_text = edit;
            }

        for _press in widget::Button::new()
            .align_middle_x()
            .label("play")
            .down_from(self.ids.input, 10.0)
            .set(self.ids.button, ui)
            {
                self.switcher.set_next(Some(Box::new(Play::new(Some(self.input_text.clone())))));
            }

        Ok(())
    }

    fn switcher(&mut self) -> &mut Switcher {
        &mut self.switcher
    }

    fn draw(&mut self, ctx: &mut Context, graphics: &mut G2d) -> GameResult<()> {
        // A function used for caching glyphs to the texture cache.
        fn cache_queued_glyphs(graphics: &mut G2d, cache: &mut G2dTexture, rect: conrod::text::rt::Rect<u32>, data: &[u8]) {
            let mut text_vertex_data = Vec::new();
            let offset = [rect.min.x, rect.min.y];
            let size = [rect.width(), rect.height()];
            let format = texture::Format::Rgba8;
            let encoder = &mut graphics.encoder;
            text_vertex_data.clear();
            text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));

            texture::UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                .expect("failed to update texture")
        }

        if let Some(primitives) = self.ui.draw_if_changed() {
            // Specify how to get the drawable texture from the image. In this case, the image
            // *is* the texture.
            fn texture_from_image<T>(img: &T) -> &T { img }

            // Draw the conrod `render::Primitives`.
            conrod::backend::piston::draw::primitives(
                primitives,
                ctx.clone(),
                graphics,
                &mut self.text_texture_cache,
                &mut self.glyph_cache,
                &self.image_map,
                cache_queued_glyphs,
                texture_from_image
            );
        }

        Ok(())
    }

    fn key_press(&mut self, button: Button) {
        if let Button::Keyboard(key) = button {
            match key {
                Key::Return => {
                    self.switcher.set_next(Some(Box::new(Play::new(None))));
                },
                _ => ()
            }
        }
    }
}