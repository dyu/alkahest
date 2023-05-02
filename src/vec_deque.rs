use alloc::collections::VecDeque;

use crate::{
    buffer::Buffer,
    bytes::Bytes,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{reference_size, Formula},
    iter::{deserialize_extend_iter, owned_iter_fast_sizes, ref_iter_fast_sizes},
    reference::Ref,
    serialize::{
        write_bytes, write_ref, write_reference, write_slice, Serialize, SerializeRef, Sizes,
    },
};

impl<F> Formula for VecDeque<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = <Ref<[F]> as Formula>::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = <Ref<[F]> as Formula>::EXACT_SIZE;
    const HEAPLESS: bool = <Ref<[F]> as Formula>::HEAPLESS;
}

impl<F, T> Serialize<VecDeque<F>> for T
where
    F: Formula,
    T: Serialize<[F]>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let size = write_ref::<[F], T, _>(self, sizes, buffer.reborrow())?;
        write_reference::<[F], B>(size, sizes.heap, sizes.heap, sizes.stack, buffer)?;
        sizes.stack += reference_size::<[F]>();
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = <Self as Serialize<[F]>>::size_hint(self)?;
        sizes.to_heap(0);
        sizes.add_stack(reference_size::<[F]>());
        Some(sizes)
    }
}

impl<'de, F, T> Deserialize<'de, VecDeque<F>> for T
where
    F: Formula,
    T: Deserialize<'de, [F]>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, DeserializeError> {
        let de = de.deref::<[F]>()?;
        <T as Deserialize<[F]>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let de = de.deref::<[F]>()?;
        <T as Deserialize<[F]>>::deserialize_in_place(self, de)
    }
}

impl<F, T> Serialize<[F]> for VecDeque<T>
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.into_iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        ref_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<F, T> SerializeRef<[F]> for VecDeque<T>
where
    F: Formula,
    for<'ser> &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let iter = de.into_unsized_iter();
        let (lower, _) = Iterator::size_hint(&iter);
        let mut vec = VecDeque::with_capacity(lower);
        deserialize_extend_iter(&mut vec, iter)?;
        Ok(vec)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.clear();
        let iter = de.into_unsized_iter();
        let (lower, _) = Iterator::size_hint(&iter);
        self.reserve(lower);
        deserialize_extend_iter(self, iter)
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let mut vec = VecDeque::with_capacity(N);
        deserialize_extend_iter(&mut vec, de.into_unsized_array_iter(N))?;
        Ok(vec)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.clear();
        self.reserve(N);
        deserialize_extend_iter(self, de.into_unsized_array_iter(N))
    }
}

impl Serialize<Bytes> for VecDeque<u8> {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let (head, tail) = self.as_slices();
        write_bytes(head, sizes, buffer.reborrow())?;
        write_bytes(tail, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl Serialize<Bytes> for &VecDeque<u8> {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let (head, tail) = self.as_slices();
        write_bytes(head, sizes, buffer.reborrow())?;
        write_bytes(tail, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl<'de> Deserialize<'de, Bytes> for VecDeque<u8> {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let bytes = de.read_all_bytes();
        let mut deque = VecDeque::with_capacity(bytes.len());
        deque.extend(bytes);
        Ok(deque)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        self.clear();
        self.extend(de.read_all_bytes());
        Ok(())
    }
}
