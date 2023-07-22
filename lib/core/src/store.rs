use std::{fs, io, marker, path};

pub struct Store<T>
where
    T: Default,
    T: Persistable,
{
    path: path::PathBuf,
    _data: marker::PhantomData<T>,
}

pub trait Persistable
where
    Self: Sized,
{
    type Error: From<io::Error> + std::fmt::Debug;

    fn read<R>(r: R) -> Result<Self, Self::Error>
    where
        R: io::Read;

    fn write<W>(&self, w: &mut W) -> Result<(), Self::Error>
    where
        W: io::Write;
}

impl<T: Default + Persistable> From<path::PathBuf> for Store<T> {
    fn from(value: path::PathBuf) -> Self {
        Store::<T>::new(value)
    }
}

impl<T: Default + Persistable> Store<T> {
    pub fn new(file: path::PathBuf) -> Store<T> {
        Store {
            path: file,
            _data: marker::PhantomData,
        }
    }

    pub fn put(&self) -> Result<T, T::Error> {
        let f = self.open_read()?;
        let item = T::read(f).or_else(|e| -> Result<T, T::Error> {
            let mut f = self.open_write()?;
            let def = T::default();
            def.write(&mut f)?;
            Ok(def)
        })?;

        Ok(item)
    }

    pub fn set(&self, item: &T) -> Result<(), T::Error> {
        let mut f = self.open_write()?;
        item.write(&mut f)
    }

    fn open_read(&self) -> io::Result<fs::File> {
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(false)
            .open(self.path.as_path())
    }

    fn open_write(&self) -> io::Result<fs::File> {
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(false)
            .truncate(true)
            .open(self.path.as_path())
    }
}
