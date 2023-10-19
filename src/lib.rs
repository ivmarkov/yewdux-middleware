use std::rc::Rc;

pub use yewdux::prelude::{Reducer, Store};

pub use self::dispatch::MiddlewareDispatch;
pub use self::functional::use_store_value;

pub trait Middleware<M, D>
where
    D: MiddlewareDispatch<M>,
{
    fn invoke(&self, msg: M, dispatch: D);
}

impl<M, L, D> Middleware<M, D> for Rc<L>
where
    L: Middleware<M, D>,
    D: MiddlewareDispatch<M>,
{
    fn invoke(&self, msg: M, dispatch: D) {
        (**self).invoke(msg, dispatch);
    }
}

impl<M, D> Middleware<M, D> for Rc<dyn Middleware<M, D>>
where
    D: MiddlewareDispatch<M>,
{
    fn invoke(&self, msg: M, dispatch: D) {
        (**self).invoke(msg, dispatch);
    }
}

impl<M, D, F> Middleware<M, D> for F
where
    D: MiddlewareDispatch<M>,
    F: Fn(M, D),
{
    fn invoke(&self, msg: M, dispatch: D) {
        (self)(msg, dispatch);
    }
}

mod functional {
    use std::rc::Rc;

    use yew::hook;
    use yewdux::store::Store;

    #[hook]
    pub fn use_store_value<S: Store>() -> Rc<S> {
        let (store, _) = yewdux::prelude::use_store();

        store
    }
}

pub mod dispatch {
    use core::cell::RefCell;

    use std::rc::Rc;

    use anymap::AnyMap;

    use super::Middleware;

    pub trait MiddlewareDispatch<M> {
        fn invoke(&self, msg: M);

        fn fuse<L>(self, middleware: L) -> CompositeDispatch<L, Self>
        where
            Self: Sized + Clone,
            L: Middleware<M, Self>,
        {
            CompositeDispatch(middleware, self)
        }
    }

    impl<M, D> MiddlewareDispatch<M> for Rc<D>
    where
        D: MiddlewareDispatch<M>,
    {
        fn invoke(&self, msg: M) {
            (**self).invoke(msg);
        }
    }

    impl<M> MiddlewareDispatch<M> for Rc<dyn MiddlewareDispatch<M>> {
        fn invoke(&self, msg: M) {
            (**self).invoke(msg);
        }
    }

    impl<M, F> MiddlewareDispatch<M> for F
    where
        F: Fn(M),
    {
        fn invoke(&self, msg: M) {
            (self)(msg);
        }
    }

    #[derive(Clone)]
    pub struct CompositeDispatch<L, D>(L, D);

    impl<M, L, D> MiddlewareDispatch<M> for CompositeDispatch<L, D>
    where
        L: Middleware<M, D>,
        D: MiddlewareDispatch<M> + Clone,
    {
        fn invoke(&self, msg: M) {
            self.0.invoke(msg, self.1.clone());
        }
    }

    pub fn void<M>(_msg: M) {}

    pub fn store<M, S>(msg: M)
    where
        M: yewdux::prelude::Reducer<S>,
        S: yewdux::prelude::Store,
    {
        yewdux::dispatch::reduce(move |state| msg.apply(state));
    }

    thread_local! {
        static REGISTRY: RefCell<AnyMap> = RefCell::new(AnyMap::new());
    }

    struct RegistryEntry<M>(Rc<dyn MiddlewareDispatch<M>>);

    pub fn invoke<M>(msg: M)
    where
        M: 'static,
    {
        get::<M>().invoke(msg);
    }

    pub fn get<M>() -> impl MiddlewareDispatch<M>
    where
        M: 'static,
    {
        let dispatch = REGISTRY.with(|registry| {
            registry
                .borrow()
                .get::<RegistryEntry<M>>()
                .map(|value| value.0.clone())
        });

        if let Some(dispatch) = dispatch {
            dispatch
        } else {
            panic!("No registered dispatch for type")
        }
    }

    pub fn register<M, D>(dispatch: D)
    where
        D: MiddlewareDispatch<M> + 'static,
        M: 'static,
    {
        REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .insert(RegistryEntry(Rc::new(dispatch)));
        });
    }
}
