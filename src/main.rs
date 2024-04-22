extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::text::{Font, TextRenderer};
use kiss3d::window::Window;
use na::{Point2, Point3};
use std::iter;
//use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::path::Path;
use std::rc::Rc;

use fps_counter::FPSCounter;

struct DigitalRain {
    size: usize,
    frame: i64,
    font: Rc<Font>,
    grid: Vec<String>,
    drips: Vec<DripAnimation>,
}

#[derive(Debug, Clone)]
struct DripAnimation {
    col: usize,
    row: usize,
    start: usize,
    velocity: i64,
    glyphs: Vec<String>,
    created: i64,
}

fn random_ascii(rng: &mut ThreadRng, len: usize) -> String {
    // ascii + half-width kana:
    //const CHARSET: &str = " !\"#$%&'`()*+,-_./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^abcdefghijklmnopqrstuvwxyz{|}~ ｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ";
    //const CHARSET: &str = " !\"#$%&'`()*+,-_./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^abcdefghijklmnopqrstuvwxyz{|}~ ｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜ";

    // customized from above to look Matrix-y
    const CHARSET: &str = " '`,-_0123456789<>?ABCDEFGHIJKLMNOPQRSTUVWXYZ ｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜ";

    //const CHARSET: &str = " ｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ";
    //const CHARSET: &str = "日本語の場合はランダムに生成された文章以外に、著作権が切れた小説などが利用されることもある。";

    let charset = CHARSET.split("").collect::<Vec<&str>>();
    let one_char = || charset[rng.gen_range(0..charset.len())];
    iter::repeat_with(one_char).take(len).collect()
}

fn random_ascii_vec(rng: &mut ThreadRng, len: usize) -> Vec<String> {
    let mut ascii = vec![];
    for x in random_ascii(rng, len).split("").collect::<Vec<&str>>() {
        if !x.is_empty() {
            ascii.push(String::from(x));
        }
    }
    println!("{:}", ascii.join(", "));
    ascii
    //let mut ascii = random_ascii(rng, len).split("").collect::<Vec<&str>>();
    //ascii.retain(|c| c != &"");
    //println!("{:}", ascii.join(", "));
    //ascii
    //    .into_iter()
    //    .map(|x| x.to_string())
    //    .collect::<Vec<String>>()
}

impl DigitalRain {
    fn update(&mut self, rng: &mut ThreadRng) {
        self.frame += 1;

        let mut deleted = vec![];
        self.drips.retain(|drip| {
            let r = drip.row < self.size;
            if !r && drip.glyphs != vec![" "] {
                deleted.push(drip.clone());
            }
            r
        });
        for deleted_drip in deleted {
            let glyph_count = rng.gen_range(1..=20);
            self.drips.push(DripAnimation {
                col: rng.gen_range(0..self.size),
                row: 0,
                start: 0,
                velocity: rng.gen_range(10..100),
                glyphs: random_ascii_vec(rng, glyph_count),
                created: self.frame,
            });
            self.drips.push(DripAnimation {
                col: rng.gen_range(0..self.size),
                row: 0,
                start: 0,
                velocity: rng.gen_range(5..50),
                glyphs: vec![" ".to_string()],
                created: self.frame,
            });
            self.drips.push(DripAnimation {
                col: deleted_drip.col,
                row: 0,
                start: 0,
                velocity: rng.gen_range(5..25),
                glyphs: vec![" ".to_string()],
                created: self.frame,
            });
        }

        for drip in self.drips.iter_mut() {
            if drip.row >= self.size {
                continue;
            }
            let advance_row = self.frame % drip.velocity == 0;
            let should_drip = self.frame % rng.gen_range(5..30) == 0;
            if should_drip || advance_row {
                self.grid[drip.col * self.size + drip.row] = drip
                    .glyphs
                    .choose(rng)
                    .unwrap_or(&" ".to_string())
                    .to_owned();
            }
            if advance_row {
                drip.row += 1;
            }
        }
    }

    fn draw(&mut self, window: &mut Window) {
        for j in 0..(self.size - 1) {
            for i in 0..(self.size - 1) {
                window.draw_text(
                    &self.grid[i * self.size + j],
                    &Point2::new(200.0 + i as f32 * 120.0, 50.0 + j as f32 * 55.0),
                    60.0,
                    &self.font,
                    &Point3::new(0.0, 1.0, 0.0),
                );
            }
        }
    }

    fn grid_at(&self, x: usize, y: usize) -> &String {
        &self.grid[y * self.size + x]
    }
    fn grid_set(&mut self, x: usize, y: usize, val: String) {
        self.grid[y * self.size + x] = val;
    }
}

fn main() {
    let width = 1200;
    let height = 820;
    let mut window = Window::new_with_size("Bitcamp 10: Digital Rain", width, height);
    let mut camera = kiss3d::planar_camera::FixedView::new();
    window.set_light(Light::StickToCamera);

    let mut fps_counter = FPSCounter::new();
    let fps_font = Font::new(&Path::new("assets/fonts/NotoSansMono_variable.ttf")).unwrap();

    let size = 38;
    let mut dr = DigitalRain {
        size: size,
        frame: 0,
        //font: Font::default(),
        //font: Font::new(&Path::new("assets/fonts/Migae.otf")).unwrap(),
        //font: Font::new(&Path::new("assets/fonts/Unifont.otf")).unwrap(),
        //font: Font::new(&Path::new("assets/fonts/NotoSansMono_variable.ttf")).unwrap(),
        //font: Font::new(&Path::new("assets/fonts/Backwards.ttf")).unwrap(),
        //font: Font::new(&Path::new("assets/fonts/Jaycons.ttf")).unwrap(),
        //font: Font::new(&Path::new("assets/fonts/NotoSansJP_variable.ttf")).unwrap(),
        font: Font::new(&Path::new("assets/fonts/NotoSansJP_backward.ttf")).unwrap(),
        //window: window,
        grid: vec![String::from(" "); size * size],
        drips: vec![],
    };

    let mut rng = rand::thread_rng();

    for _i in 0..rng.gen_range(dr.size / 4..dr.size) {
        let glyph_count = rng.gen_range(1..=20);
        dr.drips.push(DripAnimation {
            col: rng.gen_range(0..dr.size),
            row: 0,
            start: 0,
            velocity: rng.gen_range(10..100),
            glyphs: random_ascii_vec(&mut rng, glyph_count),
            created: dr.frame,
        });
    }

    //let die_zero_to_size = Uniform::from(0..dr.size);

    while window.render_with(None, Some(&mut camera), None) {
        let fps = fps_counter.tick().to_string();
        window.draw_text(
            &fps,
            &Point2::new(width as f32 * 4.0 - 45.0 * 4.0, 0.0),
            120.0,
            &fps_font,
            &Point3::new(0.0, 1.0, 1.0),
        );

        dr.update(&mut rng);
        dr.draw(&mut window);

        //if 0.9 < rng.gen::<f64>() {
        //    let x = die_zero_to_size.sample(&mut rng);
        //    let y = die_zero_to_size.sample(&mut rng);
        //    let glyph = &dr.grid[y * size + x];
        //    if glyph == "1" {
        //        dr.grid[y * size + x] = String::from("0");
        //    } else if glyph == "0" {
        //        dr.grid[y * size + x] = String::from("1");
        //    }
        //}

        for event in window.events().iter() {
            match event.value {
                WindowEvent::Close
                | WindowEvent::Key(Key::Escape, Action::Release, _)
                | WindowEvent::Key(Key::Q, Action::Release, _) => {
                    println!("close event");
                    window.close();
                }
                WindowEvent::Key(key, action, modif) => {
                    println!("key event {:?} on {:?} with {:?}", key, action, modif);
                }
                _ => {}
            }
        }
    }
}
