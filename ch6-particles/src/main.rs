use std::alloc::{GlobalAlloc, Layout, System};
use std::time::Instant;
use graphics::math::{add, sub, mul_scalar, Vec2d};
use piston_window::{clear, Context, G2d, Glyphs, PistonWindow, Position, rectangle, WindowSettings, text};
use piston_window::Key::P;
use piston_window::types::Color;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};

#[global_allocator]
static ALLOCATOR: ReportingAllocator = ReportingAllocator;
struct ReportingAllocator;


unsafe impl GlobalAlloc for ReportingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();
        let ptr = System.alloc(layout);
        let end = Instant::now();
        let time_taken = end - start;
        let bytes_requested = layout.size();

        eprintln!("{}\t{}", bytes_requested, time_taken.as_nanos());
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

struct World {
    current_turn: u64,
    particles: Vec<Box<Particle>>,
    height: f64,
    width: f64,
    rnd: ThreadRng,
}

struct Particle {
    height: f64,
    width: f64,
    position: Vec2d<f64>,
    velocity: Vec2d<f64>,
    acceleration: Vec2d<f64>,
    color: [f32; 4],
}

impl Particle {
    fn new(world: &World) -> Particle {
        let mut rng = thread_rng();

        // STarts at a random position along the top of the window
        let x = rng.gen_range(0.0..=world.width);
        let y = 0.0;

        // Raises vertically over time
        let x_velocity = 0.0;
        let y_velocity = rng.gen_range(-2.0..0.0);

        // Increases it's speed over time
        let x_acceleration = 0.0;
        let y_acceleration = rng.gen_range(0.0..0.15);

        Particle {
            height: 4.0,
            width: 4.0,
            position: [x, y].into(), // converts [f64; 2] into Vec2d
            velocity: [x_velocity, y_velocity].into(), // converts [f64; 2] into Vec2d
            acceleration: [x_acceleration, y_acceleration].into(), // converts [f64; 2] into Vec2d
            color: [1.0, 1.0, 1.0, 0.99], // Fully saturated white, with 0.01 transparency
        }
    }

    fn update(&mut self) {
        self.velocity = sub(self.velocity, self.acceleration);
        self.position = sub(self.position, self.velocity);

        // Slows down the particle as it travels.
        self.acceleration = mul_scalar(self.acceleration, 0.7);

        // Make the particle more transparent over time.

        let r = thread_rng().gen_range(0.00..0.99);
        let g = thread_rng().gen_range(0.00..0.99);
        let b = thread_rng().gen_range(0.00..0.99);

        self.color[0] = r;
        self.color[1] = g;
        self.color[2] = b;
        // self.color[3] *= 0.995;
    }
}

impl World {
    fn new(width: f64, height: f64) -> World {
        World {
            current_turn: 0,
            particles: Vec::<Box<Particle>>::new(),
            height: height,
            width: width,
            rnd: thread_rng(),
        }
    }

    fn add_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let particle = Particle::new(&self);
            let boxed_particle = Box::new(particle);
            self.particles.push(boxed_particle);
        }
    }

    fn remove_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let particle_iter = self.particles.iter().enumerate();
            let mut to_delete = None;

            // Remove the first particle that is transparent, otherwise removed the oldest particle.
            for (i, particle) in particle_iter {
                if particle.color[3] < 0.02 {
                    to_delete = Some(i);
                }
                break;
            }

            if let Some(i) = to_delete {
                self.particles.remove(i);
            }else {
                self.particles.remove(0);
            }
        }
    }

    fn update(&mut self) {
        let n = thread_rng().gen_range(-3..=3); // between -3 and 3 inclusive.

        if n > 0 {
            self.add_shapes(n);
        }else {
            self.remove_shapes(n);
        }

        self.particles.shrink_to_fit();

        for shape in &mut self.particles {
            shape.update();
        }

        self.current_turn += 1;
    }
}

fn main() {
    let (width, height) =  (1280.0, 960.0);
    let mut window: PistonWindow = WindowSettings::new("particles", [width, height])
        .exit_on_esc(true)
        .build()
        .expect("Could not create a window");

    let mut world = World::new(width, height);
    world.add_shapes(1000);

    while let Some(event) = window.next() {
        world.update();
        window.draw_2d(&event, |ctx, renderer, _device| {
            clear([0.15, 0.17, 0.17, 0.9], renderer);

            for s in &mut world.particles {
                let size = [s.position[0], s.position[1], s.width, s.height];
                rectangle(s.color, size, ctx.transform, renderer);
            }
        });
    }
}
