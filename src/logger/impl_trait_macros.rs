macro_rules! impl_log_trait {
    ($name:ty) => {
        impl ::log::Log for $name {
            fn enabled(&self, metadata: &log::Metadata) -> bool {
                <$name as $crate::logger::Logger>::enabled(
                    self,
                    metadata.level(),
                    metadata.target(),
                )
            }

            fn log(&self, record: &log::Record) {
                let _tmp_record = $crate::util::record::Record::new(record, self.name());
                <$name as $crate::logger::Logger>::log(self, _tmp_record);
            }

            fn flush(&self) {
                <$name as $crate::logger::Logger>::flush(self);
            }
        }
    };
}
