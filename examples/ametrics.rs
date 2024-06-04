use std::{thread, time::Duration};

use anyhow::Result;

use concurrency::AmapMetrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = AmapMetrics::new(&[
        "req.page.1",
        "req.page.2",
        "req.page.3",
        "req.page.4",
        "call.thread.worker.0",
        "call.thread.worker.1",
    ]);

    for i in 0..N {
        task_worker(i, metrics.clone()); // Metrics {data:Arc::clone(&metrics.data)}
    }

    for _ in 0..M {
        request_worker(metrics.clone())?;
    }

    // start N workers and M requests
    // for i in 0..100 {
    //     metrics.inc("req.page.1");
    //     metrics.inc("req.page.2");
    //     if i % 2 == 0 {
    //         metrics.inc("req.page.3");
    //     }
    // }

    // for _ in 0..30 {
    //     metrics.inc("call.thread.worker.1");
    // }

    loop {
        thread::sleep(Duration::from_secs(2));

        // println!("{:?}", metrics.snopshot());

        // 给Metrics实现Display 可以直接打印
        println!("{}", metrics);
    }

    // Ok(())
}

fn task_worker(idx: usize, metrics: AmapMetrics) {
    thread::spawn(move || loop {
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_millis(rng.gen_range(500..5000)));
        metrics.inc(format!("call.thread.worker.{}", idx)).unwrap();
    });
}

fn request_worker(metrics: AmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..1000)));
            let page = rng.gen_range(1..5);
            metrics.inc(format!("req.page.{}", page))?;
        }
        // 创建一个成功的 Result 实例
        #[allow(unreachable_code)]
        // <_, anyhow::Error> 是指定 Ok 的类型参数
        // 指定了一个单元类型 () 作为 Ok 的成功值
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}
