use bevy::{color::palettes::css::PURPLE, prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Debug, Clone, Copy, Component)]
pub struct Grid {
    pub cell_size: f32,
    pub cell_count: (u32, u32),
    pub cell_subdivision: u32,
    pub main_color: Color,
    pub subdivision_color: Color,
}

impl Grid {
    pub fn get_closest_discrete_position(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).round() as i32,
            (pos.y / self.cell_size).round() as i32,
        )
    }
    pub fn get_closest_position(&self, pos: Vec2) -> Vec2 {
        let (x, y) = self.get_closest_discrete_position(pos);
        Vec2::new(x as f32, y as f32) * self.cell_size
    }

    pub fn width(&self) -> f32 {
        self.cell_size * self.cell_count.0 as f32
    }
    pub fn height(&self) -> f32 {
        self.cell_size * self.cell_count.1 as f32
    }
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    pub fn subdivision_offset(&self) -> f32 {
        self.cell_size / self.cell_subdivision as f32
    }

    pub fn spawn(
        self,
        translation: Vec3,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        let origin = translation.truncate() - self.size() / 2.0;

        let subdivision_offset = self.subdivision_offset();

        commands
            .spawn(self)
            .insert(Transform::from_translation(translation))
            .with_children(|parent| {
                parent.spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(Rectangle::default()).into(),
                    transform: Transform::default().with_scale(Vec3::splat(128.)),
                    material: materials.add(Color::from(PURPLE)),
                    ..default()
                });

                // for i in 0..=self.cell_count.0 {
                //     let x = i as f32 * self.cell_size;
                //     parent.spawn();

                //     for j in 1..self.cell_subdivision {
                //         let x = x + j as f32 * subdivision_offset;
                //         parent.spawn(todo!());
                //     }
                // }

                // for i in 0..=self.cell_count.1 {
                //     let y = i as f32 * self.cell_size;
                //     parent.spawn(todo!());

                //     for j in 1..self.cell_subdivision {
                //         let y = y + j as f32 * subdivision_offset;
                //         parent.spawn(todo!());
                //     }
                // }
            });
    }

    // pub fn draw(&self, c: &mut RaylibMode2D<RaylibDrawHandle>, origin: Vec2) {
    //     let topleft = origin - self.size() / 2.0;

    //     let subdivision_offset = self.subdivision_offset();

    //     // c.draw_rectangle_lines(
    //     //     topleft.x as i32,
    //     //     topleft.y as i32,
    //     //     self.width() as i32,
    //     //     self.height() as i32,
    //     //     self.main_color,
    //     // );
    //     // Vertical lines
    //     for i in 0..=self.cell_count.0 {
    //         let x = i as f32 * self.cell_size;
    //         c.draw_line_v(
    //             topleft + Vec2::new(x, 0.),
    //             topleft + Vec2::new(x, self.height()),
    //             self.main_color,
    //         );

    //         for j in 1..self.cell_subdivision {
    //             let x = x + j as f32 * subdivision_offset;
    //             c.draw_line_v(
    //                 topleft + Vec2::new(x, 0.0),
    //                 topleft + Vec2::new(x, self.height()),
    //                 self.subdivision_color,
    //             );
    //         }
    //     }
    //     // Horizontal lines
    //     for i in 0..=self.cell_count.1 {
    //         let y = i as f32 * self.cell_size;
    //         c.draw_line_v(
    //             topleft + Vec2::new(0.0, y),
    //             topleft + Vec2::new(self.width(), y),
    //             self.main_color,
    //         );

    //         for j in 1..self.cell_subdivision {
    //             let y = y + j as f32 * subdivision_offset;
    //             c.draw_line_v(
    //                 topleft + Vec2::new(0.0, y),
    //                 topleft + Vec2::new(self.width(), y),
    //                 self.subdivision_color,
    //             );
    //         }
    //     }
    // }
}
