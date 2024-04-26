use anyhow::{anyhow, Result};
use std::{
    fmt,
    ops::{Add, Mul},
    sync::mpsc,
    thread,
};

const NUM_THREADS: usize = 4;

#[derive(Debug)]
pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T> Matrix<T>
where
    T: Copy,
{
    fn rows(&self) -> Vec<Vec<T>> {
        self.data.chunks(self.col).map(|x| x.to_vec()).collect()
    }

    fn cols(&self) -> Vec<Vec<T>> {
        self.rows()
            .into_iter()
            .fold(vec![vec![]; self.col], |mut cols, row| {
                for (i, item) in row.into_iter().enumerate() {
                    cols[i].push(item);
                }
                cols
            })
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix<[")?;
        for (i, row) in self.rows().iter().enumerate() {
            for (j, item) in row.iter().enumerate() {
                write!(f, "{}", item)?;
                if j < self.col - 1 {
                    write!(f, " ")?;
                }
            }

            if i < self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]>")?;
        Ok(())
    }
}

fn dot_product<T>(a: &[T], b: &[T]) -> Result<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default,
{
    if a.len() != b.len() {
        return Err(anyhow!("Length mismatch"));
    }
    Ok(a.iter()
        .zip(b.iter())
        .map(|(x, y)| *x * *y)
        .fold(T::default(), |acc, x| acc + x))
}

struct MsgInput<T> {
    idx: usize,
    rows: Vec<T>,
    cols: Vec<T>,
}

impl<T> MsgInput<T> {
    fn new(idx: usize, rows: Vec<T>, cols: Vec<T>) -> Self {
        Self { idx, rows, cols }
    }
}

struct MsgOutput<T> {
    idx: usize,
    value: T,
}

impl<T> MsgOutput<T> {
    fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> Msg<T> {
    fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let senders = (0..NUM_THREADS)
            .map(|_| {
                let (tx, rx) = mpsc::channel::<Msg<T>>();
                thread::spawn(move || {
                    let msg = rx.recv()?;
                    let value = dot_product(&msg.input.rows, &msg.input.cols)?;
                    let msg_output = MsgOutput::new(msg.input.idx, value);

                    msg.sender
                        .send(msg_output)
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

                    Ok::<_, anyhow::Error>(())
                });
                tx
            })
            .collect::<Vec<_>>();

        let max_len = self.row * rhs.col;
        let mut data = vec![T::default(); max_len];
        let mut receivers = Vec::with_capacity(max_len);

        for (i, row) in self.rows().iter().enumerate() {
            for (j, col) in rhs.cols().iter().enumerate() {
                let (tx, rx) = oneshot::channel();
                let idx = i * rhs.col + j;
                let msg_input = MsgInput::new(idx, row.clone(), col.clone());
                let msg = Msg::new(msg_input, tx);

                senders[idx % NUM_THREADS].send(msg).expect("Sender failed");
                receivers.push(rx);
            }
        }

        for rx in receivers {
            let output = rx.recv().expect("Receiver failed");
            data[output.idx] = output.value;
        }

        Self::new(data, self.row, rhs.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix() {
        let matrix = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);

        assert_eq!(matrix.rows(), vec![vec![1, 2, 3], vec![4, 5, 6]]);
        assert_eq!(matrix.cols(), vec![vec![1, 4], vec![2, 5], vec![3, 6]]);
    }

    #[test]
    fn test_dot_product() -> Result<()> {
        let matrix1 = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let matrix2 = Matrix::new([10, 11, 20, 21, 30, 31], 3, 2);

        assert_eq!(dot_product(&matrix1.rows()[0], &matrix2.cols()[0])?, 140);
        assert_eq!(dot_product(&matrix1.rows()[0], &matrix2.cols()[1])?, 146);
        assert_eq!(dot_product(&matrix1.rows()[1], &matrix2.cols()[0])?, 320);
        assert_eq!(dot_product(&matrix1.rows()[1], &matrix2.cols()[1])?, 335);

        Ok(())
    }

    #[test]
    fn test_matrix_display() {
        let matrix = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);

        assert_eq!(format!("{}", matrix), "Matrix<[1 2 3, 4 5 6]>");
    }

    #[test]
    fn test_matrix_mul() {
        let matrix1 = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let matrix2 = Matrix::new([10, 11, 20, 21, 30, 31], 3, 2);

        let result = matrix1 * matrix2;

        assert_eq!(result.rows(), vec![vec![140, 146], vec![320, 335]]);
        assert_eq!(result.cols(), vec![vec![140, 320], vec![146, 335]]);
        assert_eq!(result.row, 2);
        assert_eq!(result.col, 2);
    }
}
