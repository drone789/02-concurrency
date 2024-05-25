use std::{sync::mpsc, thread};

use anyhow::{anyhow, Result};

const NUM_PRODUCERS: usize = 4;
fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // 创建producers
    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }
    drop(tx); // 释放 tx ，否则 rx无法结束

    // 创建 customer
    let consumer = thread::spawn(move || {
        // 从通道中取 Msg
        for msg in rx {
            println!("consumer: {:?}", msg);
        }

        // 通道关闭（所有的生产者都退出）
        println!("[-] consumer exit");
        // 返回
        8817
    });

    let secret = consumer
        .join()
        .map_err(|e| anyhow!("Thread join failed: {:?}", e))?;

    println!("Secret: {:?}", secret);

    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(idx, value))?;

        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(std::time::Duration::from_millis(sleep_time));
        // random exit the producer thread
        if rand::random::<u8>() % 5 == 0 {
            println!("Producer {} exited", idx);
            break;
        }
    }
    // more thing to do
    Ok(())
}

/*
1. 多个生成者，一个消费者
2. map_err 一种错误处理的技巧:将一种错误，转成另外一种结果
*/
