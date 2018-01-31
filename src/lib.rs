//! Macros for taking ownership, _starring Liam Neeson_
//!
//! This module exports the `take!` macro which allows you to express ownership on one or more
//! variables.
//!
//! All of them expand into some sort of `let v = v;`. See the [`take!`](macro.take.html)
//! for more details and possible use cases.
//!
//! ### Special Thanks
//! This crate was created through the community efforts at [/r/rust]. Special thanks to:
//!
//! - [/u/CUViper] for poiting out the tradeoffs of this strategy.
//! - [/u/jasonkdark] for the initial implementation.
//! - [/u/i_r_witty] for improving this quote
//!
//! > _[In the voice of Liam Neeson]_
//! > But what I do have are a very particular set of macros, macros I have acquired over a very
//! > long career. Macros that make me a nightmare for closures like you. If you let my variable go
//! > now, that'll be the end of it. I will not look for you, I will not pursue you. But if you
//! > don't, I will look for you, I will find you, and I will compile you.
//!
//! [/r/rust]: https://www.reddit.com/r/rust/comments/7u29r3/help_me_make_the_own_macro_and_understand_its_use/
//! [/u/CUViper]: https://www.reddit.com/r/rust/comments/7u29r3/help_me_make_the_own_macro_and_understand_its_use/dthcvlp/
//! [/u/jasonkdark]: https://www.reddit.com/r/rust/comments/7u29r3/help_me_make_the_own_macro_and_understand_its_use/dthfcnt/
//! [/u/i_r_witty]: https://www.reddit.com/r/rust/comments/7ubwjv/announcing_the_taken_crate_with_special_thanks_to/dtjrusk/



/// Take ownership of specific variables.
///
/// You can instruct the compiler on how you want to own your variables in this way:
///
/// ```rust
/// # let (x, y) = (1, 2);
/// let x = x;
/// let y = y;
/// // ... etc
/// ```
///
/// But this is quite silly and not always completely obvious what you are trying to do.
/// Use the `take!` macro and your code will be self documenting.
///
/// ```rust
/// # #[macro_use] extern crate taken;
/// # fn main() {
/// # let (x, y) = (1, 2);
/// take!(x, y); // I see, you are just taking ownership
/// // ... etc
/// # }
/// ```
///
/// It also allows you to _change_ the mutability of variables more concisely.
///
/// ```rust
/// # #[macro_use] extern crate taken;
/// # fn main() {
/// // v has to be mutable at first
/// let mut v = vec![1, 2, 3, 4];
/// v.extend(0..100);
/// take!(v); // make sure we don't mutate v anymore.
/// # }
/// ```
///
/// As well as do operations in mass, such as if you want to assert how you own
/// variables in your closure.
///
/// ```rust
/// # #[macro_use] extern crate taken;
/// # fn main() {
/// let (mut w, x, y, z) = (42, vec![1, 2], vec![3,10], vec![4, 5, 6]);
/// {
///     let mut closure = || {
///         // Specify _how_ we are using the outside scope
///         take!(&mut w, &x, mut y, =mut z);
///
///         // We took a a mutable reference to `w`
///         *w = 777;
///
///         // `x` is an immutable reference
///         println!("we can't change x: {:?}", x);
///
///         // `y` is mutable and will be dropped at the end of scope.
///         y.push(111);
///
///         // We took a mutable clone of `z`
///         z.push(40);
///     };
///     closure();
/// }
///
/// println!("w has been mutated: {}", w);
/// println!("x couldn't change: {:?}", x);
/// // println!("y was moved: {:?}", y); // ERROR: use of moved value
/// println!("z was cloned, so it didn't change: {:?}", z);
/// # }
/// ```
///
/// One of the main use cases is closures. Closures try to be "smart" about how
/// much scope they capture. If you don't mutate a variable they take `&var`,
/// if you do mutate they take `&mut var`. However, if you require ownership
/// you use `move`, i.e. `move || {... do stuff with var...}`... _right_?
///
/// The problem with `move` is that it moves _every_ variable that is referenced. If you only need
/// to move a few variables it can be a pain. Interestingly, you can tell the compiler to only move
/// _specific_ variables using `let x = x` or `take!(x)`.
///
/// ## Downsides
/// If you use this macro (or `let x = x`) inside a closure then it becomes `FnOnce`.
///
/// Unfortunately the best explanation of the trade offs is [currently a reddit thread][reddit].
/// Please help flush out these docs more!
///
/// [reddit]: https://www.reddit.com/r/rust/comments/7u29r3/help_me_make_the_own_macro_and_understand_its_use/dthcvlp/
///
/// # Examples
///
/// ## Changing Ownership
/// It is easy to change the mutability and take references or clones on one or more variables.
///
/// ```rust
/// # #[macro_use] extern crate taken;
/// # fn main() {
/// let (a, mut b, c, d, e, f) = (1, 2, 3, 4, 5, 6);
/// take!(
///     &a,     // let a = &a;
///     &mut b, // let b = &mut b;
///     c,      // let c = c;
///     mut d,  // let mut d = d;
///     =e,     // let e = e.clone();
///     =mut f, // let mut f = f.clone();
/// );
/// # }
/// ```
///
/// ## Changing Ownership and Renaming
/// You can also rename one or more of the variables using `as`:
///
/// ```rust
/// # #[macro_use] extern crate taken;
/// # fn main() {
/// let (var_a, mut var_b, var_c, var_d, var_e, var_f) = (1, 2, 3, 4, 5, 6);
/// take!(
///     &var_a as a,     // let a = &var_a;
///     &mut var_b as b, // let b = &mut var_b;
///     var_c as c,      // let c = var_c;
///     mut var_d as d,  // let mut d = var_d;
///     =var_e as e,     // let e = var_e.clone();
///     =mut var_f as f, // let mut f = var_f.clone();
/// );
/// # }
/// ```
///
/// ## Usecase: Threads
/// Threads are another primary use case, as threads use closures. Threads in particular are always
/// `FnOnce` and often find themselves cloning and moving specific variables.
///
/// ```rust
/// use std::thread::spawn;
/// use std::sync::mpsc::channel;
/// #[macro_use] extern crate taken;
///
/// # fn main() {
/// let (send, recv) = channel();
/// {
///     let th = spawn(|| {
///         take!(send); // let send = send;
///         send.send("foo").unwrap();
///     });
///     th.join().unwrap();
/// }
/// println!("received: {:?}", recv.into_iter().collect::<Vec<_>>());
/// # }
/// ```
#[macro_export]
macro_rules! take {
    // ---------------------
    // ----- with rest -----
    [$var:ident, $($rest:tt)*] => {
        let $var = $var;
        take![$($rest)*]
    };
    [$var:ident as $v:ident, $($rest:tt)*] => {
        let $v = $var;
        take![$($rest)*]
    };

    [mut $var:ident, $($rest:tt)*] => {
        let mut $var = $var;
        take![$($rest)*]
    };
    [mut $var:ident as $v:ident, $($rest:tt)*] => {
        let mut $v = $var;
        take![$($rest)*]
    };

    [&$var:ident, $($rest:tt)*] => {
        let $var = &$var;
        take![$($rest)*]
    };
    [&$var:ident as $v:ident, $($rest:tt)*] => {
        let $v = &$var;
        take![$($rest)*]
    };

    [&mut $var:ident, $($rest:tt)*] => {
        let $var = &mut $var;
        take![$($rest)*]
    };
    [&mut $var:ident as $v:ident, $($rest:tt)*] => {
        let $v = &mut $var;
        take![$($rest)*]
    };

    [=$var:ident, $($rest:tt)*] => {
        let $var = $var.clone();
        take![$($rest)*]
    };
    [=$var:ident as $v:ident, $($rest:tt)*] => {
        let $v = $var.clone();
        take![$($rest)*]
    };

    [=mut $var:ident, $($rest:tt)*] => {
        let mut $var = $var.clone();
        take![$($rest)*]
    };
    [=mut $var:ident as $v:ident, $($rest:tt)*] => {
        let mut $v = $var.clone();
        take![$($rest)*]
    };


    // ------------------------
    // ----- without rest -----
    [$var:ident] => {
        let $var = $var;
    };
    [$var:ident as $v:ident] => {
        let $v = $var;
    };

    [mut $var:ident] => {
        let mut $var = $var;
    };
    [mut $var:ident as $v:ident] => {
        let mut $v = $var;
    };

    [&$var:ident] => {
        let $var = &$var;
    };
    [&$var:ident as $v:ident] => {
        let $v = &$var;
    };

    [&mut $var:ident] => {
        let $var = &mut $var;
    };
    [&mut $var:ident as $v:ident] => {
        let $v = &mut $var;
    };

    [=$var:ident] => {
        let $var = $var.clone();
    };
    [=$var:ident as $v:ident] => {
        let $v = $var.clone();
    };

    [=mut $var:ident] => {
        let mut $var = $var.clone();
    };
    [=mut $var:ident as $v:ident] => {
        let mut $v = $var.clone();
    };

    // trailing comma
    [] => {};
}

#[test]
#[allow(unused_mut, unused_variables, unused_assignments)]
fn sanity_syntax() {
    let x = 1;
    take!(x);
    assert_eq!(x, 1);

    take!(mut x);
    x = 2;
    assert_eq!(x, 2);

    {
        take!(&mut x);
        *x = 1;
        assert_eq!(*x, 1);
    }

    {
        take!(&x);
        assert_eq!(*x, 1);
    }

    {
        take!(=x);
        assert_eq!(x, 1);
    }

    {
        take!(=mut x);
        x = 2;
        assert_eq!(x, 2);
    }
    assert_eq!(x, 1);
}

#[test]
#[allow(unused_mut, unused_variables, unused_assignments)]
fn sanity_syntax_comma() {
    // all of these just have trailing commas, which actually tests the `rest` block as well.
    let x = 1;
    take!(x,);
    assert_eq!(x, 1);

    take!(mut x,);
    x = 2;
    assert_eq!(x, 2);

    {
        take!(&mut x,);
        *x = 1;
        assert_eq!(*x, 1);
    }

    {
        take!(&x,);
        assert_eq!(*x, 1);
    }

    {
        take!(=x,);
        assert_eq!(x, 1);
    }

    {
        take!(=mut x,);
        x = 2;
        assert_eq!(x, 2);
    }
    assert_eq!(x, 1);
}

#[test]
#[allow(unused_mut, unused_variables, unused_assignments)]
fn sanity_syntax_as() {
    let mut x = 1;
    {
        take!(x as y);
        assert_eq!(x, y);
    }

    {
        take!(mut x as y);
        y = 2;
        assert_eq!(y, 2);
    }

    {
        take!(&mut x as y);
        *y = 2;
        assert_eq!(*y, 2);
    }
    assert_eq!(x, 2);
    x = 1;

    {
        take!(&x as y);
        assert_eq!(*y, 1);
    }

    {
        take!(=x as y);
        assert_eq!(x, y);
    }

    {
        take!(=mut x as y);
        y = 2;
        assert_eq!(y, 2);
    }
    assert_eq!(x, 1);
}


#[test]
#[allow(unused_mut, unused_variables, unused_assignments)]
fn sanity_multi() {
    {
        let (x, y, z) = (1, 2, 3);
        take!(x, y, z);
    }
    {
        let (x, y, z) = (1, 2, 3);
        take!(mut x, y, z);
    }
    {
        let (x, y, z) = (1, 2, 3);
        take!(mut x, mut y, z);
    }
    {
        let (x, y, z) = (1, 2, 3);
        take!(mut x, mut y, mut z);
    }

    {
        let (x, y, z) = (1, 2, 3);
        take!(
            mut x,
            mut y,
            mut z
        );

        take!(
            &x,
            &mut y,
            z
        );
    }

    {
        // with trailing comma
        let (x, y, z) = (1, 2, 3);
        take!(
            x,
            y,
            z,
        );
    }
}
