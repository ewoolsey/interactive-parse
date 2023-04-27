use std::{
    cell::Cell,
    io::{stdout, Write},
};

use crossterm::{
    cursor::MoveToPreviousLine,
    queue,
    terminal::{Clear, ClearType},
};
use log::debug;

use crate::error::{SchemaError, SchemaResult};

pub(crate) trait Undo {
    type Output;
    fn undo(self, current_depth: &Cell<u16>) -> SchemaResult<Self::Output>;
}

impl<T> Undo for Option<T> {
    type Output = T;
    fn undo(self, current_depth: &Cell<u16>) -> SchemaResult<Self::Output> {
        let current_depth_val = current_depth.get();
        match self {
            Some(value) => {
                debug!("Depth {} -> {}", current_depth_val, current_depth_val + 1);
                current_depth.set(current_depth_val + 1);
                Ok(value)
            }
            None => {
                debug!("Undo at depth {}", current_depth_val);
                Err(SchemaError::Undo {
                    depth: current_depth_val,
                })
            }
        }
    }
}

pub(crate) trait CatchUndo {
    type Output;
    fn catch_undo(self, current_depth: &Cell<u16>) -> SchemaResult<Self::Output>;
}

impl<T> CatchUndo for Result<T, SchemaError> {
    type Output = T;
    fn catch_undo(self, current_depth: &Cell<u16>) -> SchemaResult<Self::Output> {
        let current_depth_val = current_depth.get();
        match self {
            Ok(value) => {
                debug!("Depth {} -> {}", current_depth_val, current_depth_val + 1);
                current_depth.set(current_depth_val + 1);
                Ok(value)
            }
            Err(SchemaError::Undo { depth }) => {
                debug!("Undo at depth {}", current_depth_val);
                current_depth.set(depth);
                Err(SchemaError::Undo { depth })
            }
            Err(err) => Err(err),
        }
    }
}

pub(crate) trait RecurseIter<T, U>
where
    Self: Iterator<Item = T> + Clone,
    T: Clone,
    U: Clone,
{
    fn recurse_iter<F: FnMut(T) -> SchemaResult<RecurseLoop<U>> + Clone>(
        self,
        current_depth: &Cell<u16>,
        f: F,
    ) -> SchemaResult<Vec<U>>;
}

impl<I, T, U> RecurseIter<T, U> for I
where
    I: Iterator<Item = T> + Clone,
    T: Clone,
    U: Clone,
{
    fn recurse_iter<F: FnMut(T) -> SchemaResult<RecurseLoop<U>> + Clone>(
        self,
        current_depth: &Cell<u16>,
        f: F,
    ) -> SchemaResult<Vec<U>> {
        let mut acc = Vec::new();
        recurse_loop(self, current_depth, &mut acc, f)?;
        Ok(acc)
    }
}

pub(crate) enum RecurseLoop<T> {
    Continue(T),
    Return(Option<T>),
}

pub(crate) fn recurse_loop<
    I: Iterator<Item = T> + Clone,
    T: Clone,
    U: Clone,
    F: Clone + FnMut(T) -> SchemaResult<RecurseLoop<U>>,
>(
    mut iter: I,
    current_depth: &Cell<u16>,
    acc: &mut Vec<U>,
    mut f: F,
) -> SchemaResult<()> {
    let iter_checkpoint = iter.clone();
    let acc_checkpoint = acc.clone();
    let depth_checkpoint = current_depth.get();
    let Some(item) = iter.next() else {
        return Ok(());
    };

    let val = match f(item) {
        Ok(RecurseLoop::Continue(item)) => {
            acc.push(item);
            recurse_loop(iter, current_depth, acc, f.clone())
        }
        Ok(RecurseLoop::Return(item)) => {
            if let Some(item) = item {
                acc.push(item);
            }
            Ok(())
        }
        Err(err) => Err(err),
    };

    match val {
        Err(SchemaError::Undo { depth }) => {
            if depth > depth_checkpoint {
                current_depth.set(depth_checkpoint);
                *acc = acc_checkpoint;
                clear_lines(depth - depth_checkpoint + 1);
                recurse_loop(iter_checkpoint, current_depth, acc, f)
            } else {
                Err(SchemaError::Undo { depth })
            }
        }
        other => other,
    }
}

pub(crate) fn clear_lines(n: u16) {
    let mut stdout = stdout();
    queue!(
        stdout,
        MoveToPreviousLine(n),
        Clear(ClearType::FromCursorDown)
    )
    .unwrap();
    stdout.flush().unwrap();
}
