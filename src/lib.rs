use std::rc::Rc;

use context::MiddlewareContext;
pub use yewdux::prelude::{Reducer, Store};

pub use self::dispatch::MiddlewareDispatch;
pub use self::functional::use_mcx;

pub trait Middleware<M, D>
where
    D: MiddlewareDispatch<M>,
{
    fn invoke(&self, mcx: &MiddlewareContext, msg: M, dispatch: D);
}

impl<M, L, D> Middleware<M, D> for Rc<L>
where
    L: Middleware<M, D>,
    D: MiddlewareDispatch<M>,
{
    fn invoke(&self, mcx: &MiddlewareContext, msg: M, dispatch: D) {
        (**self).invoke(mcx, msg, dispatch);
    }
}

impl<M, D> Middleware<M, D> for Rc<dyn Middleware<M, D>>
where
    D: MiddlewareDispatch<M>,
{
    fn invoke(&self, mcx: &MiddlewareContext, msg: M, dispatch: D) {
        (**self).invoke(mcx, msg, dispatch);
    }
}

impl<M, D, F> Middleware<M, D> for F
where
    D: MiddlewareDispatch<M>,
    F: Fn(&MiddlewareContext, M, D),
{
    fn invoke(&self, mcx: &MiddlewareContext, msg: M, dispatch: D) {
        (self)(mcx, msg, dispatch);
    }
}

pub mod context {
    use std::rc::Rc;

    use anymap::AnyMap;
    use yewdux::{mrc::Mrc, Context};

    use crate::MiddlewareDispatch;

    #[derive(Clone, Default, PartialEq)]
    pub struct MiddlewareContext {
        context: Context,
        registry: Mrc<AnyMap>,
    }

    impl MiddlewareContext {
        #[cfg(any(doc, feature = "doctests", target_arch = "wasm32"))]
        pub fn global() -> Self {
            thread_local! {
                static CONTEXT: MiddlewareContext = MiddlewareContext {
                    context: Context::global(),
                    registry: Mrc::new(AnyMap::new()),
                };
            }

            CONTEXT
                .try_with(|cx| cx.clone())
                .expect("CONTEXT thread local key init failed")
        }

        pub fn new() -> Self {
            Self {
                context: Context::new(),
                registry: Mrc::new(AnyMap::new()),
            }
        }

        pub fn context(&self) -> &Context {
            &self.context
        }

        pub fn void<M>(&self, _msg: M) {}

        pub fn store<M, S>(&self, msg: M)
        where
            M: yewdux::prelude::Reducer<S>,
            S: yewdux::prelude::Store,
        {
            self.context.reduce(move |state| msg.apply(state));
        }

        pub fn invoke<M>(&self, msg: M)
        where
            M: 'static,
        {
            self.get::<M>().invoke(self, msg);
        }

        pub fn get<M>(&self) -> impl MiddlewareDispatch<M>
        where
            M: 'static,
        {
            let dispatch = self
                .registry
                .borrow()
                .get::<RegistryEntry<M>>()
                .map(|value| value.0.clone());

            if let Some(dispatch) = dispatch {
                dispatch
            } else {
                panic!("No registered dispatch for type")
            }
        }

        pub fn register<M, D>(&self, dispatch: D)
        where
            D: MiddlewareDispatch<M> + 'static,
            M: 'static,
        {
            self.registry
                .borrow_mut()
                .insert(RegistryEntry(Rc::new(dispatch)));
        }
    }

    struct RegistryEntry<M>(Rc<dyn MiddlewareDispatch<M>>);
}

pub mod context_provider {
    use yew::prelude::*;

    use crate::context;

    #[derive(PartialEq, Clone, Properties)]
    pub struct Props {
        pub children: Children,
    }

    #[function_component]
    pub fn YewduxMiddlewareRoot(Props { children }: &Props) -> Html {
        let mcx = use_state(context::MiddlewareContext::new);
        html! {
            <ContextProvider<context::MiddlewareContext> context={(*mcx).clone()}>
                { children.clone() }
            </ContextProvider<context::MiddlewareContext>>
        }
    }
}

pub mod dispatch {
    use std::rc::Rc;

    use crate::context::MiddlewareContext;

    use super::Middleware;

    pub trait MiddlewareDispatch<M> {
        fn invoke(&self, mcx: &MiddlewareContext, msg: M);

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
        fn invoke(&self, mcx: &MiddlewareContext, msg: M) {
            (**self).invoke(mcx, msg);
        }
    }

    impl<M> MiddlewareDispatch<M> for Rc<dyn MiddlewareDispatch<M>> {
        fn invoke(&self, mcx: &MiddlewareContext, msg: M) {
            (**self).invoke(mcx, msg);
        }
    }

    impl<M, F> MiddlewareDispatch<M> for F
    where
        F: Fn(&MiddlewareContext, M),
    {
        fn invoke(&self, mcx: &MiddlewareContext, msg: M) {
            (self)(mcx, msg);
        }
    }

    #[derive(Clone)]
    pub struct CompositeDispatch<L, D>(L, D);

    impl<M, L, D> MiddlewareDispatch<M> for CompositeDispatch<L, D>
    where
        L: Middleware<M, D>,
        D: MiddlewareDispatch<M> + Clone,
    {
        fn invoke(&self, mcx: &MiddlewareContext, msg: M) {
            self.0.invoke(mcx, msg, self.1.clone());
        }
    }
}

mod functional {
    use yew::{hook, use_context};

    use crate::context::MiddlewareContext;

    #[hook]
    pub fn use_mcx() -> MiddlewareContext {
        #[cfg(target_arch = "wasm32")]
        {
            use_context::<crate::context::MiddlewareContext>()
                .unwrap_or_else(crate::context::MiddlewareContext::global)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use_context::<crate::context::MiddlewareContext>()
                .expect("YewduxMiddlewareRoot not found")
        }
    }
}
