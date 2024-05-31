use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use anyhow::{anyhow, Result};

use crate::{dot_product, vector::Vector};

const NUM_THREADS: usize = 4;

// [[1, 2], [1, 2], [1, 2]] => [1, 2, 1, 2, 1, 2]
pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

pub struct Msg<T> {
    input: MsgInput<T>,
    // sender to send the result back
    sender: oneshot::Sender<MsgOutput<T>>,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    if a.col != b.row {
        return Err(anyhow!("Matrix multiply err: a.col != b.row"));
    }

    // 1 遍历每一个线程
    let senders = (0..NUM_THREADS)
        .map(|_| {
            // 2 为每一个线程生成通道(rx,tx)
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                // 3 遍历每个线程通道的rx(接受)
                for msg in rx {
                    // 4 处理
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    // 5 发送回结果
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            // 6 闭包返回tx(发送)
            tx
        })
        .collect::<Vec<_>>(); // 7. 将所有线程的tx汇总到集合中

    // generate 4 threads which receive msg and do dot product
    let matrix_len = a.row * b.col;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    // map/reduce: map phase
    for i in 0..a.row {
        for j in 0..b.col {
            // for k in 0..a.col {
            //     data[i * b.col + j] += a.data[i * a.col + k] * b.data[k * b.col + j];
            // }

            // 矩阵的一行[..]
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);

            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            // 矩阵的一列
            let col = Vector::new(col_data);

            let idx = i * b.col + j;
            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel::<MsgOutput<T>>();

            // 组装成Msg
            let msg = Msg::new(input, tx);

            // 随机选择一个通道,发送
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Send error: {:?}", e);
            }

            // 接受通道汇总
            receivers.push(rx);
        }
    }

    // map/reduce: reduce phase
    for rx in receivers {
        let output = rx.recv()?;
        data[output.idx] = output.value;
    }
    // 返回最终结果
    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

impl<T: fmt::Debug> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    // display a 2x3 as {1 2 3, 4 5 6}, 3x2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }

            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiply error")
    }
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a * b;
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");
        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _c = a * b;
    }
}

/*

senders
[
            mpsc::channel::<Msg<T>>                                  oneshot
            ________________                                      ______________
    tx -> ()________________()  rx --> 线程1处理---结果发送到通道---()______________()---
                                                                                     |
             ________________                                     ______________
    tx -> ()________________()  rx --> 线程2处理---结果发送到通道---()______________()   | -->data

            _________________                                     ______________
    tx -> ()________________()  rx --> 线程3处理---结果发送到通道---()______________()   |

            _________________                                     ______________
    tx -> ()________________()  rx --> 线程4处理---结果发送到通道---()______________()---|
]

*/
