use crate::{sink::Sink, util::DEFAULT_NULL_NAME};

impl<T: Sink, const N: usize> Sink for [T; N] {
    #[inline(always)]
    fn log(&self, record: &crate::Record) -> crate::sink::SinkResult<()> {
        for s in self {
            s.log(record)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn flush(&self) -> crate::sink::SinkResult<()> {
        for s in self {
            s.flush()?;
        }
        Ok(())
    }

    #[inline(always)]
    fn name(&self) -> &str {
        self.first().map(|s| s.name()).unwrap_or(DEFAULT_NULL_NAME)
    }

    #[inline(always)]
    fn set_level(&self, level: log::LevelFilter) {
        for s in self {
            s.set_level(level);
        }
    }
}

macro_rules! impl_for_tuple {
    ($($idx:tt : $T:ident),+) => {
        impl<$($T: Sink),+> Sink for ($($T,)+) {
            #[inline]
            fn log(&self, record: &crate::Record) -> crate::sink::SinkResult<()> {
                // 逐个转发，遇到 Err 立即用 ? 提前返回
                $(self.$idx.log(record)?;)+
                Ok(())
            }

            #[inline]
            fn flush(&self) -> crate::sink::SinkResult<()> {
                $(self.$idx.flush()?;)+
                Ok(())
            }

            /// 实现将返回 self.0.name()。
            #[inline]
            fn name(&self) -> &str {
                // 返回第一个元素的名字作为元组的名字
                self.0.name()
            }

            #[inline]
            fn set_level(&self, level: log::LevelFilter) {
                $(self.$idx.set_level(level);)+
            }
        }
    };
}

// 空元组：什么都不做即可
impl Sink for () {
    #[inline(always)]
    fn log(&self, _record: &crate::Record) -> crate::sink::SinkResult<()> {
        Ok(())
    }

    #[inline(always)]
    fn flush(&self) -> crate::sink::SinkResult<()> {
        Ok(())
    }

    #[inline(always)]
    fn name(&self) -> &str {
        DEFAULT_NULL_NAME
    }

    #[inline(always)]
    fn set_level(&self, _level: log::LevelFilter) {}
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
