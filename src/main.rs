use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::cell::RefCell;
use std::marker::PhantomData;

/// Arc wrapper.
struct MyArc<T>(Arc<T>);

/// keep same as Arc
unsafe impl<T> Send for MyArc<T> where
    T: Send + Sync {}

/// remove `T：Send`
unsafe impl<T> Sync for MyArc<T> where
    T: Sync {}

/// Construct a struct Witch `Sync + !Send`
struct A(PhantomData<*const ()>);
unsafe impl Sync for A {}

impl A {
    fn new() -> Self {
        println!("A created at thread: {:?}", thread::current().id());
        A(PhantomData)
    }
}

impl Drop for A {
    fn drop(&mut self) {
        println!("A dropped at thread: {:?}", thread::current().id())
    }
}

thread_local! {
    static FOO: RefCell<Option<MyArc<A>>> = RefCell::new(None);
}

fn main() {
    let a = MyArc(Arc::new(A::new()));
    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();
    pool.scope(|s| {
        s.spawn(|_| {
            let b = MyArc(Arc::clone(&a.0));
            FOO.with(move |cell|{
                println!("setting MyArc to thradlocal, thread id: {:?}", thread::current().id());
                *cell.borrow_mut() = Some(b);
            });
        });
    });

    
    drop(a);
    drop(pool);

    // wait for rayon's threads to drop
    thread::sleep(Duration::from_secs(3));
}
