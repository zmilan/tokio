use crate::{runtime, sync::mpsc, task, util};
use futures::stream::StreamExt;

#[test]
fn drop_mpsc() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                let tx = tx;
                println!("task 2 yielding");
                task::yield_now().await;
                println!("drop tx");
            });
            task::spawn(async move {
                println!("task 1 await rx");
                rx.recv().await;
                println!("drop rx");
            });
            task::spawn(task::yield_now()).await;
            println!("thread 1: after spawn");
        });
        drop(rt);
        println!("rt dropped");
        println!("thread joined");
    });
}

#[test]
fn drop_mpsc2() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                println!("task 1 await rx");
                rx.recv().await;
                println!("drop rx");
            });
            task::spawn(task::yield_now()).await;
            println!("thread 1: after spawn");
        });
        drop(tx);
        println!("drop tx");
        drop(rt);
        println!("rt dropped");
        println!("thread joined");
    })
}

#[test]
fn drop_mpsc3() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                let tx = tx;
                println!("task 2 yielding");
                task::yield_now().await;
                println!("drop tx");
            })
            .await;
            task::spawn(async move {
                println!("task 1 await rx");
                rx.recv().await;
                println!("drop rx");
            });
            task::spawn(task::yield_now()).await;
            println!("thread 1: after spawn");
        });
        drop(rt);
        println!("rt dropped");
        println!("thread joined");
    })
}

#[test]
fn drop_mpsc4() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                let tx = tx;
                println!("task 2 yielding");
                task::yield_now().await;
                println!("drop tx");
            });
            task::spawn(async move {
                println!("task 1 await rx");
                rx.recv().await;
                println!("drop rx");
            })
            .await;
            task::spawn(task::yield_now()).await;
            println!("thread 1: after spawn");
        });
        drop(rt);
        println!("rt dropped");
        println!("thread joined");
    })
}

#[test]
fn drop_mpsc5() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                let mut tx = tx;
                tx.send(()).await;
                println!("(drop_mpsc5): task 2 yielding");
                task::yield_now().await;
                println!("(drop_mpsc5): drop tx");
            });
            task::spawn(async move {
                println!("(drop_mpsc5): task 1 await rx");
                rx.recv().await;
                println!("(drop_mpsc5): task 1 await rx again");
                rx.recv().await;
                println!("(drop_mpsc5): drop rx");
            })
            .await;
            task::spawn(task::yield_now()).await;
            println!("(drop_mpsc5): thread 1: after spawn");
        });
        drop(rt);
        println!("(drop_mpsc5): rt dropped");
        println!("(drop_mpsc5): thread joined");
    })
}

#[test]
fn drop_mpsc6() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                println!("(drop_mpsc6): task 1 await rx");
                rx.recv().await;
                println!("(drop_mpsc6): task 1 await rx again");
                rx.recv().await;
                println!("(drop_mpsc6): drop rx");
            });
            task::spawn(task::yield_now()).await;
            println!("(drop_mpsc6): thread 1: after spawn");
        });
        tx.send(());
        println!("(drop_mpsc6): sent");
        drop(rt);
        println!("(drop_mpsc6): rt dropped");
        println!("(drop_mpsc6): thread joined");
    })
}

#[test]
fn drop_mpsc7() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        let tx = rt.block_on(async move {
            task::spawn(async move {
                println!("(drop_mpsc7): task 1 await rx");
                rx.recv().await;
                println!("(drop_mpsc7): task 1 await rx again");
                rx.recv().await;
                println!("(drop_mpsc7): drop rx");
            });
            println!("(drop_mpsc7): thread 1: after spawn");
            task::spawn(task::yield_now()).await;

            tx.send(()).await;
            println!("(drop_mpsc7): thread 1: after send");
            tx
        });
        println!("(drop_mpsc7): sent");
        drop(tx);
        println!("(drop_mpsc7): drop tx");
        drop(rt);
        println!("(drop_mpsc7): rt dropped");
        println!("(drop_mpsc7): thread joined");
    })
}

#[test]
fn drop_mpsc8() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        let tx = rt.block_on(async move {
            task::spawn(async move {
                println!("(drop_mpsc8): task 1 await rx");
                rx.recv().await;
                println!("(drop_mpsc8): task 1 await rx again");
                rx.recv().await;
                println!("(drop_mpsc8): drop rx");
            });
            println!("(drop_mpsc8): thread 1: after spawn");
            task::spawn(async move {
                println!("(drop_mpsc8): task 2 send");
                tx.send(()).await;
                println!("(drop_mpsc8): task 2 after send");
                loop {
                    task::yield_now().await;
                }
            });
            task::spawn(task::yield_now()).await;

            println!("(drop_mpsc8): thread 1: after send");
        });
        println!("(drop_mpsc8): drop tx");
        drop(rt);
        println!("(drop_mpsc8): rt dropped");
        println!("(drop_mpsc8): thread joined");
    })
}

#[test]
fn drop_mpsc9() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        let tx = rt.block_on(async move {
            let tx2 = tx.clone();
            task::spawn(async move {
                println!("(drop_mpsc9): task 2 send");
                tx.send(()).await;
                println!("(drop_mpsc9): task 2 after send");
                loop {
                    task::yield_now().await;
                }
            });
            task::spawn(async move {
                println!("(drop_mpsc9): task 3 start");
                let tx2 = tx2;
                println!("(drop_mpsc9): task 3 loop");
                loop {
                    task::yield_now().await;
                }
            });
            task::spawn(async move {
                println!("(drop_mpsc9): task 1 await rx");
                rx.recv().await;
                println!("(drop_mpsc9): task 1 await rx again");
                rx.recv().await;
                println!("(drop_mpsc9): drop rx");
            });
            println!("(drop_mpsc9): thread 1: after spawn");
            task::spawn(task::yield_now()).await;

            println!("(drop_mpsc9): thread 1: after send");
        });
        println!("(drop_mpsc9): drop tx");
        drop(rt);
        println!("(drop_mpsc9): rt dropped");
        println!("(drop_mpsc9): thread joined");
    })
}

#[test]
fn drop_mpsc10() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                println!("(drop_mpsc10): task 2 send");
                let tx = tx;
                loop {
                    task::yield_now().await;
                }
            });
            task::spawn(async move {
                rx.recv().await;
            });
        });
        drop(rt);
        println!("(drop_mpsc10): rt dropped");
        println!("(drop_mpsc10): thread joined");
    })
}

#[test]
fn drop_mpsc11() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                println!("(drop_mpsc11): task 2 send");
                let tx = tx;
                loop {
                    task::yield_now().await;
                }
            });
            task::spawn(async move {
                rx.recv().await;
            });
            task::spawn(task::yield_now()).await;
        });
        drop(rt);
        println!("(drop_mpsc11): rt dropped");
        println!("(drop_mpsc11): thread joined");
    })
}

#[test]
fn drop_mpsc12() {
    util::test::with_timeout(std::time::Duration::from_secs(60), || {
        let (mut tx, mut rx) = mpsc::channel::<()>(1);
        let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
        rt.block_on(async move {
            task::spawn(async move {
                println!("(drop_mpsc11): task 2 send");
                let tx = tx;
                drop(tx);
                loop {
                    task::yield_now().await;
                }
            });
            rx.for_each(|_| async {}).await;
        });
        drop(rt);
        println!("(drop_mpsc11): rt dropped");
        println!("(drop_mpsc11): thread joined");
    })
}

#[test]
fn drop_oneshot() {
    let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
    rt.block_on(async {
        let (tx, rx) = crate::sync::oneshot::channel::<()>();
        task::spawn(task::yield_now()).await;
        task::spawn(async move {
            let tx = tx;
            loop {
                task::yield_now().await;
            }
        });
        task::spawn(async move {
            rx.await;
        });
    });
    drop(rt);
}
// #[test]
// fn drop_mpsc2() {
//     loom::model(|| {
//         let (rx, mut tx) = mpsc::channel::<()>(1);
//         let mut rt = runtime::Builder::new().basic_scheduler().build().unwrap();
//         rt.block_on(async move {
//             task::spawn(async move {
//                 task::yield_now().await;
//                 println!("task 2: after yield");
//                 tx.recv().await;
//                 println!("task 2: after recv");
//             });
//             task::spawn(task::yield_now()).await;
//             task::spawn(async move {
//                 let rx = rx;
//                 loop {
//                     task::yield_now().await;
//                     println!("yielded");
//                 }
//             });
//             println!("dropped rx");
//             println!("end block on");
//         });
//     })
// }
