//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::{RefCell, RefMut};
use core::marker::PhantomData;
use core::ops::DerefMut;

use lock_api::{Mutex, MutexGuard, RawMutex};

pub trait Shared {
    type Container<T>: Clone;

    type ContainerInit;

    type ExclusiveAccess<'a, T: 'a>: DerefMut<Target = T>;

    fn new_with<T>(init: Self::ContainerInit, value: T) -> Self::Container<T>;

    fn new<T>(value: T) -> Self::Container<T>;

    fn lock<'a, T>(shared: &'a Self::Container<T>) -> Self::ExclusiveAccess<'a, T>;
}

pub struct SharedRcRefCell(());

impl Shared for SharedRcRefCell {
    type Container<T> = Rc<RefCell<T>>;

    type ContainerInit = ();

    type ExclusiveAccess<'a, T: 'a> = RefMut<'a, T>;

    fn new_with<T>(_init: Self::ContainerInit, value: T) -> Self::Container<T> {
        Self::new(value)
    }

    fn new<T>(value: T) -> Self::Container<T> {
        Rc::new(RefCell::new(value))
    }

    fn lock<'a, T>(shared: &'a Self::Container<T>) -> Self::ExclusiveAccess<'a, T> {
        shared.borrow_mut()
    }
}

pub struct SharedArcMutex<R>(PhantomData<R>);

// R: 'static is a hack
impl<R: RawMutex + 'static> Shared for SharedArcMutex<R> {
    type Container<T> = Arc<Mutex<R, T>>;

    type ContainerInit = R;

    type ExclusiveAccess<'a, T: 'a> = MutexGuard<'a, R, T>;

    fn new_with<T>(init: Self::ContainerInit, value: T) -> Self::Container<T> {
        Arc::new(Mutex::from_raw(init, value))
    }

    fn new<T>(value: T) -> Self::Container<T> {
        Arc::new(Mutex::new(value))
    }

    fn lock<'a, T>(shared: &'a Self::Container<T>) -> Self::ExclusiveAccess<'a, T> {
        shared.lock()
    }
}
