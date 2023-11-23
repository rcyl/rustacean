struct Grounded;
struct Launched;

struct Rocket<Stage> {
    stage: std::marker::PhantomData<Stage>,
}

impl Default for Rocket<Grounded> {
    fn default() -> Self { 
        Rocket {
            stage: std::marker::PhantomData,
        }
    }
}

impl Rocket<Grounded> {
    pub fn launch(self) -> Rocket<Launched> {
        Rocket {
            stage: std::marker::PhantomData,
        }
    }
}

impl Rocket<Launched> {
    pub fn accelerate(&mut self) {}
    pub fn decelerate(&mut self) {}
}

impl <Stage> Rocket <Stage> {
    pub fn colour(&self) -> String { String::new() }
    pub fn weight(&self) -> i32 { 1000 }
}

pub mod lib {
    #[non_exhaustive]
    pub struct Unit { 
        pub field: bool,
        private: bool, 
    }
}

/// assert!(1 == 1);
/// 
pub fn first() {}

// fn is_true(u: lib::Unit) -> bool {  
//     matches!(u, lib::Unit { field: true} )
// }

#[test]
fn this_should_fail() {
    let mut x = 42;
    let x: *mut i32 = &mut x;
    let (x1, x2) = unsafe { (&mut *x, &mut *x) };
    println!("{} {}", x1, x2);
}

use intro_lib::{fibonacci, vecpush};

fn main() {
    let ans = fibonacci(4);
    println!("{}", ans);
    vecpush();
}
    



