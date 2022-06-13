use egui::{ColorImage, ImageButton, ScrollArea, TextureHandle, Ui, Vec2};

use super::palette_to_map;
use crate::ui::import_image_dialog;
use gamercade_core::{ColorIndex, Palette, SpriteIndex, SpriteSheet};

#[derive(Clone, Default)]
pub struct SheetEditor {
    pub selected_sprite: SpriteIndex,
    raw_rgba_buffer: Vec<u8>,
    texture_handles: Vec<TextureHandle>,
}

impl SheetEditor {
    pub fn draw(&mut self, ui: &mut Ui, sheet: &mut SpriteSheet, scale: f32, palette: &Palette) {
        let step = sheet.width * sheet.height * 4;
        self.raw_rgba_buffer.clear();

        ui.group(|ui| {
            ui.label("Sprite Sheet Editor");
            ui.label(format!("Sprite Count: {}", sheet.count));

            // Draws all the sprites
            // TODO: Make this tiled to fill up space correctly
            ScrollArea::both().show(ui, |ui| {
                ui.group(|ui| {
                    sheet
                        .iter_sprites()
                        .enumerate()
                        .for_each(|(index, sprite)| {
                            let start = step * index;
                            let end = start + step;
                            let index = SpriteIndex(index as u8);

                            sprite.iter().for_each(|color_index| {
                                let rgba = palette[*color_index].into_pixel_data();
                                self.raw_rgba_buffer.extend(rgba);
                            });

                            let rgb = ColorImage::from_rgba_unmultiplied(
                                [sheet.width, sheet.height],
                                &self.raw_rgba_buffer[start..end],
                            );

                            let image = match self.texture_handles.get_mut(index.0 as usize) {
                                Some(handle) => {
                                    handle.set(rgb);
                                    handle
                                }
                                None => {
                                    self.texture_handles.push(
                                        ui.ctx().load_texture(format!("sprite_{}", index.0), rgb),
                                    );
                                    &mut self.texture_handles[index.0 as usize]
                                }
                            };

                            let button = ImageButton::new(
                                image,
                                Vec2 {
                                    x: sheet.width as f32 * scale,
                                    y: sheet.height as f32 * scale,
                                },
                            )
                            .selected(self.selected_sprite == index);

                            if ui.add(button).clicked() {
                                self.selected_sprite = index;
                            };
                        });
                });
            });

            // TODO: add editor buttons:
            // New, Copy, Move Left, Move Right
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("New").clicked() {
                        sheet.new_empty(self.selected_sprite);
                        self.selected_sprite = SpriteIndex(self.selected_sprite.0 + 1);
                    }

                    if ui.button("Copy").clicked() {
                        sheet.duplicate(self.selected_sprite);
                        self.selected_sprite = SpriteIndex(self.selected_sprite.0 + 1);
                    }

                    if ui.button("Color Swap").clicked() {
                        println!("TODO: Color Swap")
                    }

                    if ui.button("Move Left").clicked() {
                        println!("TODO: Move Left")
                    }

                    if ui.button("Move Right").clicked() {
                        println!("TODO: Move Right")
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        if sheet.count != 1 {
                            sheet.delete_sprite(self.selected_sprite);

                            if self.selected_sprite.0 == sheet.count {
                                self.selected_sprite.0 = self.selected_sprite.0 - 1;
                            };
                        } else {
                            println!("Can't delete last sprite!")
                        }
                    }

                    if ui.button("Import").clicked() {
                        match try_load_sprite(sheet, palette) {
                            Ok(new_sprite) => {
                                sheet.add_new_sprite(self.selected_sprite, &new_sprite);
                                self.selected_sprite = SpriteIndex(self.selected_sprite.0 + 1);
                            }
                            Err(e) => println!("{}", e),
                        }
                    }

                    if ui.button("Export").clicked() {
                        println!("TODO: Export")
                    }
                });
            });
        });
    }
}

fn try_load_sprite(sheet: &SpriteSheet, palette: &Palette) -> Result<Box<[ColorIndex]>, String> {
    // File opening stuff
    let (image, _) = match import_image_dialog("Import Sprite...") {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    // Check if dimensions match
    if sheet.width as u32 != image.width() || sheet.height as u32 != image.height() {
        return Err("Imported image width and height don't match the sprite sheet.".to_string());
    }

    // Build the colors map, and load the sprite
    let colors = palette_to_map(palette);
    let mut output = Vec::with_capacity(image.len());

    for color in image.pixels() {
        if let Some(index) = colors.get(color) {
            output.push(*index)
        } else {
            return Err(format!(
                "Image contains a color not found in the palette: {:?}",
                color
            ));
        }
    }

    Ok(output.into_boxed_slice())
}
