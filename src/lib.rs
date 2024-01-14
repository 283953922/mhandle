pub type BoxFuture<'a, Output> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Output> + Send + 'a>>;

pub trait Handle<'a, Context>
where
    Self: Send + Sync + 'static,
{
    type Output;

    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output>;
}

impl<'a, Context, Output, F, Fut> Handle<'a, Context> for F
where
    F: Fn(&'a mut Context) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Output> + Send + 'a,
    Context: 'a,
{
    type Output = Output;
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Box::pin((self)(cx))
    }
}

#[cfg(test)]

mod tests {
    use anyhow::Error;
    use futures::executor::block_on;
    use std::sync::Arc;

    use crate::{BoxFuture, Handle};

    type Result = anyhow::Result<()>;
    type Middleware = dyn for<'a> Handle<'a, Context, Output = Result>;

    struct Context {
        index: usize,
        middleware: Vec<Arc<Middleware>>,
    }

    impl Context {
        async fn do_next(&mut self) -> Result {
            if let Some(mw) = self.middleware.pop() {
                mw.call(self).await
            } else {
                Ok(())
            }
        }
    }

    async fn fa(cx: &mut Context) -> Result {
        println!(
            "exec fn fa--{} >> index:{}",
            "--".repeat(cx.middleware.len()),
            cx.index
        );
        println!("begin call next middleware");
        let fut = cx.do_next().await;
        println!("called next middleware");
        println!(
            "exec fn fa--{} >> index:{}",
            "--".repeat(cx.middleware.len()),
            cx.index
        );
        fut
    }
    #[allow(clippy::needless_lifetimes)]
    fn fb<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result> {
        println!(
            "exec fn fb--{} >> index:{}",
            "--".repeat(cx.middleware.len()),
            cx.index
        );
        println!("begin call next middleware");
        cx.index += 1;
        Box::pin(async move {
            let fut = cx.do_next().await;
            println!(
                "exec fn fb--{} >> index:{}",
                "--".repeat(cx.middleware.len()),
                cx.index
            );
            fut
        })
    }

    #[allow(clippy::needless_lifetimes)]
    fn fc<'a>(cx: &mut Context) -> BoxFuture<'_, Result> {
        println!(
            "exec fn fc--{} >> index:{}",
            "--".repeat(cx.middleware.len()),
            cx.index
        );
        println!("begin call next middleware");
        cx.index += 1;
        Box::pin(async move {
            let fut = cx.do_next().await;
            println!(
                "exec fn fc--{} >> index:{}",
                "--".repeat(cx.middleware.len()),
                cx.index
            );
            fut
        })
    }

    #[derive(Clone)]
    struct Sa {
        index: usize,
    }

    impl<'a> Handle<'a, Context> for Sa {
        type Output = Result;

        fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
            Box::pin(async move {
                println!(
                    "exec St Sa--{} >> index:{}",
                    "--".repeat(cx.middleware.len()),
                    cx.index
                );
                println!("begin call next middleware");
                let fut = cx.do_next().await;
                println!("called next middleware");
                println!(
                    "exec St Sa--{} >> index:{}",
                    "--".repeat(cx.middleware.len()),
                    cx.index
                );
                fut
            })
        }
    }

    #[derive(Clone)]
    struct Sb {
        index: usize,
    }

    impl<'a> Handle<'a, Context> for Sb {
        type Output = Result;

        fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
            Box::pin(async move {
                println!(
                    "exec St Sb--{} >> index:{}",
                    "--".repeat(cx.middleware.len()),
                    cx.index
                );
                println!("begin call next middleware");
                let fut = cx.do_next().await;
                println!("called next middleware");
                println!(
                    "exec St Sb--{} >> index:{}",
                    "--".repeat(cx.middleware.len()),
                    cx.index
                );
                fut
            })
        }
    }

    #[derive(Clone)]
    struct Sc {
        index: usize,
    }

    impl<'a> Handle<'a, Context> for Sc {
        type Output = Result;

        fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
            Box::pin(async move {
                println!(
                    "exec St Sc--{} >> index:{}",
                    "--".repeat(cx.middleware.len()),
                    cx.index
                );
                println!("begin call next middleware");
                let fut = cx.do_next().await;
                println!("called next middleware");
                println!(
                    "exec St Sc--{} >> index:{}",
                    "--".repeat(cx.middleware.len()),
                    cx.index
                );
                fut
            })
        }
    }

    #[test]
    fn futures_runtime() {
        let result = block_on(async move {
            let mut middlewares: Vec<Arc<Middleware>> = vec![];
            let my_middleware = |cx: &mut Context| {
                println!("we handle it");
                async move { Ok(()) }
            };

            middlewares.push(Arc::new(my_middleware));
            middlewares.push(Arc::new(Sc { index: 3 }));
            middlewares.push(Arc::new(Sb { index: 2 }));
            middlewares.push(Arc::new(Sa { index: 1 }));
            middlewares.push(Arc::new(fc));
            middlewares.push(Arc::new(fb));
            middlewares.push(Arc::new(fa));
            let mut cx = Context {
                index: 0,
                middleware: Vec::new(),
            };
            cx.middleware = middlewares.clone();
            println!("mw 1: {}", middlewares.len());

            let result = cx.do_next().await?;
            assert_eq!(result, ());

            cx.middleware = middlewares.clone();
            println!("mw 2: {}", middlewares.len());

            let result = cx.do_next().await?;
            assert_eq!(result, ());
            cx.middleware = middlewares.clone();
            println!("mw 3: {}", middlewares.len());

            let result = cx.do_next().await?;
            assert_eq!(result, ());
            Ok::<_, Error>(())
        });

        assert!(result.is_ok())
    }
}
