## repr
```
repr(transparent)
```
Can only be used with types with a single field which guarantees that
the layout of the outer type is exactly the same as that of the inner type. 
This is handy with the "newtype" pattern, where you want the memory representation
of sturct A and struct NewA(A) to be the same

```repr(packed)``` is self explanatory

```repr(align(n))``` to ensure that different value is stored contiguously in memory

## Dispatch
Use static dispath in libraries and dynamic dispatch in binaries

## Associated Type
```
trait Foo<T> vs Foo { type Bar; }
```
Use associated type if you expect only one implementation of the trait for a given type
and generic type parameter otherwise. 

## Orphan rule
You can implement a trait for a type only if the trait OR the type is local to your crate

## Higher Ranked trait bound
```
F: for<'a> Fn(&'a T) -> &'a U
```
For any lifetime 'a, the bound must hold. Generally ythe compiler adds this for you
so the explicit form is exceedingly rare. 

## Borrow vs Deref/AsRef
Borrow is for your type is essentially equivalent to another type, whereas
Deref and AsRef is for anything your type can "act as"

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
unlikely to add much to the size of the overall Result type they are contained within
Error types should also be 'static so that it can be propagated up the call stack
without running into lifetime issues. 

## Error downcasting
Downcasting allows a user turn a dyn Error into a concrete underlying error type
If the user wants to perform an action if the error was std::io::Error (with 
std::io::ErrorKind::WouldBlock), and they get a dyn Error, they can use 
Error::downcast_ref to downcast the error into a std::io::Error. Downcast_ref
only works if the argument is 'static.

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

## Operating system options
#[cfg(any(windows,target_os = "macos"))]

Conditional dependencies
```
[target.'cfg(windows)'.dependencies]
winrt = "0.7"
[target.'cfg(unix)'.dependencies]
nix = "0.17"
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
to mimic what they contain, so they are ran as integration tests. 

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

## Size of generators
The data structure used to back a generator’s state must be able to hold the combined state at any one yield point. If your async fn contains, say, a [u8; 8192], those 8KiB must be stored in the generator itself. The means the code can get quite large without any visible indicators. 
copying. In fact, you can usually identify when the size of your generator-based futures is affecting performance by looking for excessive amounts of time spent in the memcpy function in your application’s performance profiles. 
When you do find a particularly large future, you have two options: you can try to reduce the amount of local state the async functions need, or you can move the future to the heap (with Box::pin) so that moving the future just requires moving the pointer to it. The latter is by far the easiest way to go, but it also introduces an extra allocation and a pointer indirection.

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

## Unsafe fn versus unsafe code block
unsafe fn is an unsigned contract that asks the author of calling code to 
“solemnly swear X, Y, and Z.” Meanwhile, unsafe {} is the calling code’s 
author signing off on all the unsafe contracts contained within the block.

## Pointer types
Consider using std::ptr::NonNull<T> if you know the pointer is never null, it is
analogous to a &. If the pointer might be null, use *const T. Raw pointers 
(*const T and *mut T) do not have lifetimes. 

You can cast any Rust pointer to a *const std:ffi::c_void or *mut std::ffi::c_void

## Opting out of safety checks
Some safe implementations include bounds check that either panic or return an Option
if the index provided is out of bounds. However, this adds overhead and may not be
acceptable in high performing code. When peak performance is important and the caller
knows that the indexes are in bound, many data structures provide alternate versions 
of particular methods without the safety checks. The usually include the world unchecked
and dont have those slow safety checks, for example slice::get_unchecekd, Arc::get_mut_unchecked

## Send and Sync
A common mistake with unsafe implementations of Send and Sync is to forget to add bounds to generic parameters: unsafe impl<T: Send> Send for MyUnsafeType<T> {}.

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
zeros first, ie [0; 4096]. We allow the array to keep whatever values happend 
to be on the stack when the function was called. 
This optimization could be crucial for hot loops.

## Casting
```
struct Foo<T> {
    one: bool,
    two: PhantomData<T>,
}

struct Bar;
struct Baz;

type A = Foo<Bar>;
type B = Foo<Baz>;
```
Rust does not guarantee that A & B have the same in memory representation.
The lack of guarantees in repr(Rust) means we must be careful when casting
in unsafe code.

## False sharing
On Intel, cache line size is 64 bytes, which means every operation reads/writes
some multiple of 64 bytes. False sharing happens when two cores want to update
the value of two different bytes that happen to fall on the same cache line. 
The updates execute sequentially, though they are logically disjoint. To avoid this,
pad your values so that they are the size of a cache line. 

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

## Acquire and Release 
- Acquire -> load
- Release -> store
- AcqReq -> load and store (like fetch_add)

Rules
1. Load and stores cannot be moved forward past a store with Ordering::Release.
2. Load and stores cannot be moved back before a load with Ordering::Acquire
3. Ordering::Acquire load of a variable must see all stores that happened before 
an Ordering::Release store that stored what the load loaded

```
static X: AtomicBool = AtomicBool::new(false);
static Y: AtomicBool = AtomicBool::new(false);

let t1 = spawn(|| {
    let r1 = Y.load(Ordering::Acquire);
    X.store(r1, Ordering::Release);
});

let t2 = spawn(|| {
    let r2 = X.load(Ordering::Acquire); (1)
    Y.store(true, Ordering::Release); (2)
});
```
The first rule prevents the reordering or (1) and (2) and prevents earlier
scenario from happening

On x86, there is no additional cost to using Ordering::Release/Acquire over
Ordering::Relaxed. This is not the case for other architectures and your programs
might be faster if you use Relaxed for atomic operations that can tolerate weaker
memory ordering guarantees. 

## Loom
Consider loom for concurrency testing

## Running TSan
RUSTFLAGS="-Z sanitizer=$SAN" cargo test --target x86_64-unknown-linux-gnu
Where $SAN is one of address, leak, memory or thread.

## Quirky c types
C types like __be32, don't often translate directly to Rust types and may be
best left as something like [u8; 4]. For example __be32 is encoded as big-endian
whereas Rust's i32 follows the endianess of the current platform

## FFI memory allocation
- Implementation managed memory interface. Memory is managed by rust usually 
via Box for example fn new() -> *mut RustType
type and a free function that deallocates the memory when done.
- Caller managed memory. Caller calls fn new(my_mem: *const c_void, len: c_int) -> *mut RustType
No free function is provided as deallocation happens in the caller. 

Prefer to allow caller to pass in memory when it is feasible since it gives the 
caller more freedom to manage memory as it deems appropriate. Caller might be using 
a highly specialized allocator or some custom OS and may not want to be forced
to use the standard allocator. Caller could even use stack or reuse already allocated
memory. 

Go with caller allcoated memory for anything that is large or frequent. 

## Include generated bindings
If bindgen runs in build.rs and generate bindings.rs, it can be included with
the following
```
include!(concat!(env!("OUT_DIR"), "/bindings/rs"));
```

## Include assembly files *.S 
To include assembly files, can consider having a build.rs that compiles the .S file 
into an object and then packages it into a static archive .a using something like ar before
linking it via "cargo:rustc-link-lib=static=xyz" in the build file. 

## Rust runtime
There's in some special code that run before main and be can be considered a 
bare-bones runtime such as the panic handler. 
Panic handler invokes the panic hook set via set::panic::set_hook.
Not all targets, especially embedded ones, provide a panic handler.  

The first thing that runs in Rust is not main, but a standard library call 
***lang_start***. It setups the rust runtime, including 

1) stashing the program's command line arguments (so that std::env::args can get to them), 
2) setting the name of the main thread
3) handling panics in the main function
4) flushing standard output on program exit
5) setting up signal handlers 

#![no_main] attrribute, completey omits lang_start, which means the developer
must figure out how the program should be started such as by declaring a function
with #[export_name = "main"]

## Out of memory handler
The default behavior of the out-of-memory handler on std-enabled platforms is to print an error message to standard error and then abort the process. However, on a platform that, for example, doesn’t have standard error, that obviously won’t work. At the time of writing, on such platforms your program must explicitly define an out-of-memory handler using the unstable attribute #[lang = "oom"].

## Low level low memory access
It is common for hardware devices have memory-mapped registers that are
modified when they are read, meaning the read have side effects. 
Rust provides volatile memory operations that cannot be elided or reordered with respect to other volatile operations. These operations take the form of std::ptr::read_volatile and std::ptr::write_volatile.

# Cargo tools
- cargo-deny: Lint dependency graph
- cargo-expand: Expand macro
- cargo-hack: Check if crate works with any combination of features enabled
- cargo-llvm-lines: Analyze mapping from Rust Code to the IR and tells you which 
bit produce the largest IR. Largest IR means longer compile times so knowing
which code generates a bigger IR can present opportunities for reducing compile times
- cargo-outdated - Check if dependencies have newer version available
- cargo-udeps - Check for unused dependencies

## Libraries
- bytes: Efficent mechanism for passing subslices of single piece of contiguous 
memory without having to deal with lifetimes. Common in low level networking code
- criterion: Statistics based benchmarking library
- flume: MPMC (multiple produce, multiple consumer) channel that is faster and 
simpler than std library's and supports both async and sync
- hdrhistogram: High dynamic range histogram
- heapless: Provides data structures that do not use the heap, perfect for embedded
- itertools: extends Iterator trait from std library. Can reduce boiler plate, so example
checking if an iterator has exactly one time (Itertools::exactly_one)
- nix: Idiomatic bindings to system calls on Unix-like systems
- pin-project: Avoid the hassle of getting Pin and Unpin right for your own types
- slab: Efficent data structure in use in place of HashMap<Token, T> where
Token is an opaque type used only to differentiate between entries in the map
- static_assertions: assertions that are evaluated as compile time
- structopt: Provides a way to describe your application's command line entirely
using the Rust type system

## Configuring cargo to share build artifacts
Set [build] target in ~/.cargo/config.toml to the directory shared artifacts should go in. 
Note that this can cause problems for projects that assume that compiler artifacts
will always be under the target subdirectory

## Check build timings
- cargo build --timings

## Print out the sizes of all the types and alignment in the current crate. 
- RUSTFLAGS=-Zprint-type-sizes cargo +nightly build --release
- rustc +nightly -Zprint-type-sizes input.rs

## Iter::once
The iter::once function takes any value and produces an iterator that yields that value once. This comes in handy when calling functions that take iterators if you don’t want to allocate, or when combined with Iterator::chain to append a single item to an existing iterator.

## std::sync Once
Runs a given piece of code exactly once at initialization time. Good for FFI

## Methods
- Arc::make_mut: Takes a &mut Arc&lt;T&gt; and gives you a &mut T
- Clone::clone_from: Alternative to .clone() that lets you reuse and instance 
of the type you clone rather than allocate a new one. Performs copy assignment from source
- Instant:elapsed: Returns the Duration since ance Instant was created
- Option::as_deref: Takes an Option&lt;P&gt; where P: Deref and returns Option<&P:Target>
this can make functional transfromation chains cleaner by avoiding .as_ref().map(|r| &**r)
- Ord::clamp: x.clamp(min, max) returns min if x is less than min, max if x is greater and max
and x otherwise
- Result::transpose (or Option::transpose): inverts types that nest Result and Option. 
When combined with ? can make for cleaner code when working with Iterator::next()
- Vec::swap_remove is faster than Vec::remove because it swaps the to be removed element
with the last element and trucates the vector's length by 1

## Patterns in the wild
- indexmap: HashMap implementation where iteration order matches the map insertion order. 
this uses index pointers

## Drop guard
```
fn mutex(lock: &AtomicBool, f: impl FnOnce()) {
    // .. while lock.compare_exchange(false, true).is_err() ..
    struct DropGuard<'a>(&'a AtomicBool);
    impl Drop for DropGuard<'_> {
        fn drop(&mut self) {
            lock.store(true, Ordering::Release);
        }
    }
    let _guard = DropGuard(lock);
    f();
}
```
This code ensures that the cleanup code gets run even when f() panics
Although the guard is never refered to it again, it needs a name because
let _ = DropGuard(lock) would drop the guard immediatedly


