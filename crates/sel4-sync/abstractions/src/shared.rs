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

pub trait Shared: 'static {
    type Container<T>: SharedContainer<T>;
}

pub trait SharedContainer<T>: Clone {
    type Init;

    const DEFAULT_INIT: Self::Init;

    type Guard<'a>: DerefMut<Target = T>
    where
        Self: 'a,
        T: 'a;

    fn new(init: Self::Init, value: T) -> Self;

    fn lock<'a>(&'a self) -> Self::Guard<'a>;
}

pub struct SharedRcRefCell(());

impl Shared for SharedRcRefCell {
    type Container<T> = Rc<RefCell<T>>;
}

impl<T> SharedContainer<T> for Rc<RefCell<T>> {
    type Init = ();

    const DEFAULT_INIT: Self::Init = ();

    type Guard<'a> = RefMut<'a, T> where Self: 'a, T: 'a;

    fn new(_init: Self::Init, value: T) -> Self {
        Rc::new(RefCell::new(value))
    }

    fn lock<'a>(&'a self) -> Self::Guard<'a> {
        self.borrow_mut()
    }
}

pub struct SharedArcMutex<R>(PhantomData<R>);

impl<R: RawMutex + 'static> Shared for SharedArcMutex<R> {
    type Container<T> = Arc<Mutex<R, T>>;
}

impl<R: RawMutex + 'static, T> SharedContainer<T> for Arc<Mutex<R, T>> {
    type Init = R;

    const DEFAULT_INIT: Self::Init = R::INIT;

    type Guard<'a> = MutexGuard<'a, R, T> where Self: 'a, T: 'a;

    fn new(init: Self::Init, value: T) -> Self {
        Arc::new(Mutex::from_raw(init, value))
    }

    fn lock<'a>(&'a self) -> Self::Guard<'a> {
        Mutex::lock(self)
    }
}
