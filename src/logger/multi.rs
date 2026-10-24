use crate::{logger::Logger, sink::Sink, util::DEFAULT_NULL_NAME};

/// 可以将泛型参数设置为元组从而绑定多个logger。
#[repr(transparent)]
pub struct MultiLogger<T>(pub T);

impl<T> MultiLogger<T> {
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl Logger for MultiLogger<()> {
    #[inline]
    fn add_sink(&self, _sink: std::sync::Arc<dyn Sink>) {}

    #[inline]
    fn remove_sink(&self, _name: String) {}

    #[inline]
    fn log_once(&self, _record: crate::util::Record) {}

    #[inline]
    fn flush_and_wait(&self) {}

    #[inline]
    fn name(&self) -> &str {
        DEFAULT_NULL_NAME
    }
}

impl log::Log for MultiLogger<()> {
    #[inline]
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    #[inline]
    fn log(&self, _record: &log::Record) {}

    #[inline]
    fn flush(&self) {}
}

macro_rules! impl_for_tuple {
    ($($idx:tt : $T:ident),+) => {
        impl<$($T: Logger),+> Logger for MultiLogger<($($T,)+)> {
            #[inline]
            fn add_sink(&self, sink: std::sync::Arc<dyn Sink>) {
                $(self.0.$idx.add_sink(sink.clone());)+
            }

            #[inline]
            fn remove_sink(&self, name: String) {
                $(self.0.$idx.remove_sink(name.clone());)+
            }

            #[inline]
            fn log_once(&self, record: crate::util::Record) {
                $(self.0.$idx.log_once(record.clone());)+
            }

            #[inline]
            fn flush_and_wait(&self) {
                $(self.0.$idx.flush_and_wait();)+
            }

            /// 返回第一个名字。
            #[inline]
            fn name(&self) -> &str {
                self.0.0.name()
            }
        }

        impl<$($T: Logger),+> log::Log for MultiLogger<($($T,)+)> {
            #[inline]
            fn enabled(&self, metadata: &log::Metadata) -> bool {
                $(self.0.$idx.enabled(metadata) &&)+ true
            }

            #[inline]
            fn log(&self, record: &log::Record) {
                $(self.0.$idx.log(record);)+
            }

            #[inline]
            fn flush(&self) {
                $(self.0.$idx.flush();)+
            }
        }
    };
}

impl_for_tuple!(0: T0);
impl_for_tuple!(0: T0, 1: T1);
impl_for_tuple!(0: T0, 1: T1, 2: T2);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14, 15: T15);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14, 15: T15, 16: T16);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14, 15: T15, 16: T16, 17: T17);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14, 15: T15, 16: T16, 17: T17, 18: T18);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14, 15: T15, 16: T16, 17: T17, 18: T18, 19: T19);
impl_for_tuple!(0: T0, 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6, 7: T7, 8: T8, 9: T9, 10: T10, 11: T11, 12: T12, 13: T13, 14: T14, 15: T15, 16: T16, 17: T17, 18: T18, 19: T19, 20: T20);
