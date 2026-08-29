macro_rules! impl_format {
    ({$($self:tt)*}, $start_color_range:path, $end_color_range:path) => {
        #[inline]
        fn format(&self, record: &crate::util::Record) -> String {
            let mut buf = crate::util::dispatcher::acquire_string();
            let sys_time =
                SystemTime::UNIX_EPOCH + Duration::from_nanos_u128(record.time_stamp_nano());
            let time =
                time::OffsetDateTime::from_unix_timestamp_nanos(record.time_stamp_nano() as i128)
                    .unwrap()
                    .to_offset(time::UtcOffset::current_local_offset().unwrap());

            for pattern in self$($self)*.pattern.read().unwrap().iter() {
                match pattern {
                    PatternCharacter::Void => unreachable!(), // 已经在初始化时排除。
                    PatternCharacter::Message => {
                        let _ = write!(buf, "{}", record.log_detail());
                    }
                    PatternCharacter::LoggerName => {
                        buf.push_str(record.logger_name());
                    }
                    PatternCharacter::LevelLower => {
                        buf.push_str(LOG_LEVEL_NAMES_LOWER[record.level() as usize]);
                    }
                    PatternCharacter::LevelUpper => {
                        buf.push_str(record.level().as_str());
                    }
                    PatternCharacter::ThreadId => {
                        let _ = write!(buf, "{:?}", record.thread_id());
                    }
                    PatternCharacter::ProcessId => {
                        let _ = write!(buf, "{}", record.process_id());
                    }
                    PatternCharacter::SourceLocation => {
                        buf.push_str(record.module_path().unwrap_or("?"));
                    }
                    PatternCharacter::SourceFile => {
                        buf.push_str(record.file().unwrap_or("?"));
                    }
                    PatternCharacter::SourceShortFile => {
                        buf.push_str(record.file().unwrap_or("?"));
                    }
                    PatternCharacter::Line => {
                        if let Some(line) = record.line() {
                            let _ = write!(buf, "{}", line);
                        } else {
                            buf.push('?');
                        }
                    }
                    PatternCharacter::FuncName => unimplemented!(),
                    PatternCharacter::Year4 => {
                        let _ = write!(buf, "{}", YEAR4[time.year() as usize - 1970]);
                    }
                    PatternCharacter::Year2 => {
                        let _ = write!(buf, "{}", PAD2[(time.year() % 100) as usize]);
                    }
                    PatternCharacter::Month => {
                        let _ = write!(buf, "{}", PAD2[time.month() as usize]);
                    }
                    PatternCharacter::Day => {
                        let _ = write!(buf, "{}", PAD2[time.day() as usize]);
                    }
                    PatternCharacter::Hour24 => {
                        let _ = write!(buf, "{}", PAD2[time.hour() as usize]);
                    }
                    PatternCharacter::Hour12 => {
                        let _ = write!(buf, "{}", PAD2[time.hour().saturating_sub(12) as usize]);
                    }
                    PatternCharacter::Minute => {
                        let _ = write!(buf, "{}", PAD2[time.minute() as usize]);
                    }
                    PatternCharacter::Second => {
                        let _ = write!(buf, "{}", PAD2[time.second() as usize]);
                    }
                    PatternCharacter::Millisecond => {
                        let _ = write!(buf, "{}", PAD3[time.millisecond() as usize]);
                    }
                    PatternCharacter::Microsecond => {
                        let _ = write!(buf, "{}", PAD3[time.microsecond() as usize % 1000]);
                    }
                    PatternCharacter::Nanosecond => {
                        let _ = write!(buf, "{}", PAD3[(time.nanosecond() % 1000) as usize]);
                    }
                    PatternCharacter::AMPM => {
                        if time.hour() >= 12 {
                            let _ = write!(buf, "PM");
                        } else {
                            let _ = write!(buf, "AM");
                        }
                    }
                    PatternCharacter::TimezoneOffset => {
                        let _ = write!(buf, "{}", time.offset());
                    }
                    PatternCharacter::UnixTimestamp => {
                        let _ = write!(buf, "{}", time.unix_timestamp());
                    }
                    PatternCharacter::StandardDateTime => unreachable!(), // 在初始化阶段就被替换。
                    PatternCharacter::ElapsedMicroseconds => {
                        let _ = write!(
                            buf,
                            "{:06}",
                            sys_time
                                .duration_since(self$($self)*.last_parse.load())
                                .unwrap_or(Duration::ZERO)
                                .as_micros()
                        );
                    }
                    PatternCharacter::ElapsedNanoseconds => {
                        let _ = write!(
                            buf,
                            "{:09}",
                            sys_time
                                .duration_since(self$($self)*.last_parse.load())
                                .unwrap_or(Duration::ZERO)
                                .as_nanos()
                        );
                    }
                    PatternCharacter::StartColorRange => $start_color_range(&self, &mut buf, record),
                    PatternCharacter::StopColorRange => $end_color_range(&mut buf),
                    PatternCharacter::Literal(c) => {
                        buf.push(*c);
                    }
                    PatternCharacter::AllMappedDiagnosticContext => {
                        unimplemented!("将会围绕 log 库的 kv 模块实现。")
                    }
                    PatternCharacter::DefaultFormat => unreachable!(), // 在初始化阶段就被替换。
                }
            }

            self$($self)*.last_parse.store(sys_time);
            buf
        }
    };
}
