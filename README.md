## Moving value behing a mutable reference
```
fn replace_with_84(s: &mut Box<i32>) {
    //let was = *s;
    let was = std::mem::take(s);
    *s = was;
    let mut r = Box::new(84);
    std::mem::swap(s, &mut r);
    assert_ne!(*r, 84);
}
```
## Generic parameter to restrict methods 

```
struct Grounded;
struct Launched;

struct Rocket<Stage> {
    // We only needed the meta-data implied by the stage
    // and not the actual struct, so we store as PhantomData
    // to guarantee it is elimited at compile time
    stage: std::marker::PhantomData<Stage>,
}

// Rocket::default() only returns Rocket<Grounded>
impl Default for Rocket<Grounded> {
    fn default() -> Self { 
        Rocket {
            stage: std::marker::PhantomData,
        }
    }
}

// After launching, Rocket<Grounded> -> Rocket<Launched>
impl Rocket<Grounded> {
    pub fn launch(self) -> Rocket<Launched> {
        Rocket {
            stage: std::marker::PhantomData,
        }
    }
}

// These methods only availabe to launched rockets
impl Rocket<Launched> {
    pub fn accelerate(&mut self) {}
    pub fn decelerate(&mut self) {}
}

// These methods are available to all stages
impl <Stage> Rocket <Stage> {
    pub fn colour(&self) -> String { String::new() }
    pub fn weight(&self) -> i32 { 1000 }
}

fn main() {
    let rocket: Rocket<Grounded> = Rocket::default();
    rocket.colour();
    let mut rocket: Rocket<Launched> = rocket.launch();
    rocket.accelerate();
    rocket.weight();
}
```

## Errors 
In genernal, the comunity consensus is that errors are rare and therefore
should not add much cost to the "happy path". For that reason, errors are 
often placed behind a pointer type, such a Box or Arc, so that they are 
unlikely to add much to the size of the overall Result type thy're contained within

## Overiding dependency sources
``` 
# Cargo.toml

[patch.crates-io]
# Use a local (probably modified) source
regex = { path = "/home/common/regex }

# use a modificaton on a git branch
serde = { git = "https://github.com/serde-rs/serde.git", branch="faster"}

# patch a git dependency
[patch.'https://github.com/somebody/project.git']
project = { path = "/home/somebody/project" }
```

## Use cargo deny and cargo audit
Use on CI to detect dependency on unmaintained crates or that those that have
known security vulnerabilities, or licenses that you want to avoid

## Versioning
- Breaking changes -> Major version change
- Additions -> Minor version change
- Bug fixes -> Patch version change

## Minimal dependency versions 
Progammers commonly choose the latest version, or just the current major version
but chances are both of those choices are wrong. Not it then sense that you 
crate won't compile, but causes strife for the users of your crate down the line.
Eg, if you add a dependency on hugs = "1.7.3". However some user also depeneds on your
crate that also depends on the another crate foo that depends on hugs. But in foo,
the dependency on hugs is "1, <1.6". Cargo sees that hugs = "1.7.3" so it considers
versions >= 1.7. But it also sees foo's dependency on hugs which is > 1.6, so Cargo
gives up and reports that there is no version of hugs compatible with all the 
requirements. 

The right strategy is to list the earliest version that has all the things your 
crate depends on and to make sure that this remains the case even as you add new 
code to your crate. But how do you establish that beyond trawling the changelogs, 
or through trial and error? Your best bet is to use Cargo’s unstable 
-Zminimal-versions flag, which makes your crate use the minimum acceptable version 
for all dependencies, rather than the maximum. Then, set all your dependencies 
to just the latest major version number, try to compile, and add a minor version 
to any dependencies that don’t. Rinse and repeat until everything compiles fine, 
and you now have your minimum version requirements!

It’s worth noting that, like with MSRV, minimal version checking faces an 
ecosystem adoption problem. While you may have set all your version specifiers 
correctly, the projects you depend on may not have. This makes the Cargo minimal 
versions flag hard to use in practice (and is why it’s still unstable).

## Cfg test
Use #[cfg(test)] to create setters/getters to access fields on structs for testing
purposes as opposed to setting the field pub or using pub(crate)

You can also use #[cfg(test)] to execute specific lines of code for example
incrementing the number of times a particular function is called, that is useful
for assertion purposes in test

## Doc tests
Doc tests appear in the public documentation of the crate and users are likely
to mimic what they contain, so they are ran as integration testts. 

## Linting
Consider using cargo clippy in your CI. Clippy can catch code patterns that compile 
but are almost certainly bugs. Some examples are a = b; b = a, 
which fails to swap a and b; std::mem::forget(t), where t is a reference; 
and for x in y.next(), which will iterate only over the first element in y.

## Miri
```
let mut x = 42;
let x: *mut i32 = &mut x;
let (x1, x2) = unsafe { (&mut *x, &mut *x) };
println!("{} {}", x1, x2);
```
This code is problematic because it crates two exclusive references to a value
(in unsafe code) which can be caught by miri

```
rustup +nightly component add miri
cargo +nightly miri test/run 
```
## Loom
Consider the crate loom for concurrent checking and atomic stuff

## Criterion
This crate runs a function a number of times to be sure that the result is
reliable.

## Disabling compiler optimization
```
let mut vs = Vec::with_capacity(4);
let start = std::time::Instant::now();
for i in 0..4 {
  vs.push(i);
}
println!("took {:?}", start.elapsed());
```
Checking with compiler explorer or cargo-asm you might notice that the calls
to Vec::with_capacity and Vec::push have been optimized out completely. 
The compiler might see that no subsequent operationss are needed on the vector so 
it optimized it away. To prevent this optimization, consider using 
std::hint::black_box (Revisit when have context)

## Fused Future
When a future has returned Poll::Ready, you should not poll a future again. If 
you do, the future is well within its rights to panic. A future that is safe to poll
after it has returned Ready is referred to as a fused future. 

## Generators
Briefly described, a generator is a chunk of code with some extra compiler-generated 
bits that enables it to stop, or yield, its execution midway through and then 
resume from where it last yielded later on.

## Pin
Though Future makes use of Pin, Pin is not tied to the Future trait - you can
use Pin for any self-refential data structure (ie, one that holds both data and
references to that data, like the ones created by generators)

Pin holds a pointer type. Rather than have a Pin<MyType>, you'll have
a Pin<Box<MyType>> or Pin<Rc<MyType>> or Pin<&mut MyType>. This is because once
you place a T behind a Pin, that T won't move. 

Use pin_mut! macro to pin a T to the stack, but mostly use Box to pin on heap instead

## Waker
A future that does not poll other futures but does something like write to a 
network socket or attempt to receive on a channel as known as leaf futures
as they have no children

Waking is a misnomer since it tells the executor to poll a particular future when
it gets around to it rather than sleeping. This might wake the executor if it is
currently sleeping but that's more of a side effect than a primary purpose. 

Rule of thumb: no future should be able to run for more than 1ms without returning
Poll::Pending

## Pointer types
Consider using std::ptr::NonNull<T> if you know the pointer is never null, it is
analogous to a &. If the pointer might be null, use *const T. Raw pointers 
(*const T and *mut T) do not have lifetimes. 

You can cast any Rust pointer to a *const std:ffi::c_void or *mut std::ffi::c_void

## Opting out of safety checks
Some safe implementations include bounds check that either panic or return an Option
if the index provided is out of bounds. However, this adds overhead and may not be
acceptable in high performaing code. When peak performance is important and the caller
knows that the indexes are in bound, many data structures provide alternate versions 
of particular methods without the safety checks. The usually include the world unchecked
and dont have those slow safety checks, for example slice::get_unchecekd, Arc::get_mut_unchecked

## Maybe Uninit for hot loop
```
fn fill(gen: impl FnMut() -> Option<u8>) {
    let mut buf = [MaybeUninit::<u8>::uninit(); 4096];
    let mut last  = 0;
    for (i, g) in std::iter::from_fn(gen).take(4096).enumerate() {
        buf[i] = MaybeUnint::new(g);
        last = i + 1;
    }
    let init: &[u8] = unsafe {
        MaybeUninit::slice_assume_init_ref(&buf[..last])
    };
}
```
This function allows us to fill in array without explicity setting the array to
zeros first, ie [0; 4096]. This optimization could be crucial for hot loops

## Relaxed memory ordering
```
static X: AtomicBool = AtomicBool::new(false);
static Y: AtomicBool = AtomicBool::new(false);

let t1 = spawn(|| {
    let r1 = Y.load(Ordering::Relaxed); (1)
    X.store(r1, Ordering::Relaxed); (2)
});

let t2 = spawn(|| {
    let r2 = X.load(Ordering::Relaxed); (3)
    Y.store(true, Ordering::Relaxed); (4)
});

```
It may look that is it unlikely for r2 to be true since we expect the following 
to happen:
1. r1 is set to false by Y.load (1)
2. X is set to false to X.store(r1) (2)
3. r2 is then set to false by X.load (3)

Though actually it is entirely possible.

Note that (4) does not have to happen after (3) since (4) doesn't use any output
or side effect of (3), ergo (4) has no dependency on (3). So the CPU can reorder them
and execute (4) first instead of (3)

The scenario that can lead to r2 being true is:
1. Y is set to true by Y.store(true) (4)
2. t2 is put to sleep and t1 runs instead
3. In t1, (1) must run first since (2) depends on the value read in (1), ie
X.store(r1,...) depends on r1
4. r1 is set to true by Y.load (1)
5. X is set to true by X.store(r1) (2)
6. t2 wakes up and sets r2 to true by X.load() (3)