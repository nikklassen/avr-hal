//! Serial Implementations

/// Serial Error
#[derive(Debug, Clone, Copy)]
pub enum Error { }

/// Implement serial traits for a USART peripheral
#[macro_export]
macro_rules! impl_usart {
    (
        $(#[$usart_attr:meta])*
        pub struct $Usart:ident {
            peripheral: $USART:ty,
            pins: {
                rx: $rxmod:ident::$RX:ident,
                tx: $txmod:ident::$TX:ident,
            },
            registers: {
                control_a: $control_a:ident {
                    data_empty: $dre:ident,
                    recv_complete: $rxc:ident,
                },
                control_b: $control_b:ident {
                    tx_enable: $txen:ident,
                    rx_enable: $rxen:ident,
                },
                control_c: $control_c:ident {
                    mode: $umode:ident,
                    char_size: $csz:ident,
                    stop_bits: $sbs:ident,
                    parity: $par:ident,
                },
                baud: $baud:ident,
                data: $data:ident,
            },
        }
    ) => {
        $(#[$usart_attr])*
        pub struct $Usart<CLOCK, RX_MODE>
        where
            CLOCK: $crate::clock::Clock,
            RX_MODE: $crate::port::mode::InputMode,
        {
            p: $USART,
            rx: $rxmod::$RX<$crate::port::mode::Input<RX_MODE>>,
            tx: $txmod::$TX<$crate::port::mode::Output>,
            _clock: ::core::marker::PhantomData<CLOCK>,
        }

        impl<CLOCK, RX_MODE> $Usart<CLOCK, RX_MODE>
        where
            CLOCK: $crate::clock::Clock,
            RX_MODE: $crate::port::mode::InputMode,
        {
            /// Initialize the USART peripheral
            ///
            /// Please note that not all baudrates will produce a good signal
            /// and setting it too high might make data sent completely unreadable
            /// for the other side.
            pub fn new(
                p: $USART,
                rx: $rxmod::$RX<$crate::port::mode::Input<RX_MODE>>,
                tx: $txmod::$TX<$crate::port::mode::Output>,
                baud: u32,
            ) -> $Usart<CLOCK, RX_MODE> {
                // Calculate BRR value
                let brr = CLOCK::FREQ / (16 * baud) - 1;
                // Set baudrate
                p.$baud.write(|w| unsafe { w.bits(brr as u16) });
                // Enable receiver and transmitter
                p.$control_b
                    .write(|w| w.$txen().set_bit().$rxen().set_bit());
                // Set frame format (8n1)
                p.$control_c.write(|w| {
                    w.$umode()
                        .usart_async()
                        .$csz()
                        .chr8()
                        .$sbs()
                        .stop1()
                        .$par()
                        .disabled()
                });

                $Usart {
                    p,
                    rx,
                    tx,
                    _clock: ::core::marker::PhantomData,
                }
            }
        }

        $crate::paste::item! {
            impl<CLOCK, RX_MODE> $Usart<CLOCK, RX_MODE>
            where
                CLOCK: $crate::clock::Clock,
                RX_MODE: $crate::port::mode::InputMode,
            {
                /// Helper method for splitting this read/write object into two halves.
                ///
                /// The two halves returned implement the `Read` and `Write` traits, respectively.
                pub fn split(self) -> ([<Read $Usart>]<CLOCK, RX_MODE>, [<Write $Usart>]<CLOCK>) {
                    (
                        [<Read $Usart>] {
                            p: unsafe { ::core::ptr::read(&self.p) },
                            rx: self.rx,
                            _clock: self._clock,
                        },
                        [<Write $Usart>] {
                            p: self.p,
                            tx: self.tx,
                            _clock: self._clock,
                        }
                    )
                }
            }
        }

        impl<CLOCK, RX_MODE> $crate::hal::serial::Write<u8> for $Usart<CLOCK, RX_MODE>
        where
            CLOCK: $crate::clock::Clock,
            RX_MODE: $crate::port::mode::InputMode,
        {
            type Error = $crate::serial::Error;

            fn write(&mut self, byte: u8) -> $crate::nb::Result<(), Self::Error> {
                // Call flush to make sure the data-register is empty
                self.flush()?;

                self.p.$data.write(|w| unsafe { w.bits(byte) });
                Ok(())
            }

            fn flush(&mut self) -> $crate::nb::Result<(), Self::Error> {
                if self.p.$control_a.read().$dre().bit_is_clear() {
                    Err($crate::nb::Error::WouldBlock)
                } else {
                    Ok(())
                }
            }
        }

        impl<CLOCK, RX_MODE> $crate::ufmt::uWrite for $Usart<CLOCK, RX_MODE>
        where
            CLOCK: $crate::clock::Clock,
            RX_MODE: $crate::port::mode::InputMode,
        {
            type Error = $crate::serial::Error;

            fn write_str(&mut self, s: &str) -> ::core::result::Result<(), Self::Error> {
                use $crate::prelude::*;

                for b in s.as_bytes().iter() {
                    $crate::nb::block!(self.write(*b))?;
                }
                Ok(())
            }
        }

        impl<CLOCK, RX_MODE> $crate::hal::serial::Read<u8> for $Usart<CLOCK, RX_MODE>
        where
            CLOCK: $crate::clock::Clock,
            RX_MODE: $crate::port::mode::InputMode,
        {
            type Error = $crate::serial::Error;

            fn read(&mut self) -> $crate::nb::Result<u8, Self::Error> {
                if self.p.$control_a.read().$rxc().bit_is_clear() {
                    return Err($crate::nb::Error::WouldBlock);
                }

                Ok(self.p.$data.read().bits())
            }
        }

        $crate::paste::item! {
            /// The readable half of the
            $(#[$usart_attr])*
            pub struct [<Read $Usart>]<CLOCK, RX_MODE>
            where
                CLOCK: $crate::clock::Clock,
                RX_MODE: $crate::port::mode::InputMode,
            {
                p: $USART,
                rx: $rxmod::$RX<$crate::port::mode::Input<RX_MODE>>,
                _clock: ::core::marker::PhantomData<CLOCK>,
            }

            /// The writable half of the
            $(#[$usart_attr])*
            pub struct [<Write $Usart>]<CLOCK>
            where
                CLOCK: $crate::clock::Clock,
            {
                p: $USART,
                tx: $txmod::$TX<$crate::port::mode::Output>,
                _clock: ::core::marker::PhantomData<CLOCK>,
            }

            impl<CLOCK, RX_MODE> [<Read $Usart>]<CLOCK, RX_MODE>
            where
                CLOCK: $crate::clock::Clock,
                RX_MODE: $crate::port::mode::InputMode,
            {
                /// Puts the two "halves" of a split `Read + Write` back together.
                pub fn reunite(self, other: [<Write $Usart>]<CLOCK>) -> $Usart<CLOCK, RX_MODE> {
                    $Usart {
                        p: self.p,
                        rx: self.rx,
                        tx: other.tx,
                        _clock: self._clock,
                    }
                }
            }

            impl<CLOCK> [<Write $Usart>]<CLOCK>
            where
                CLOCK: $crate::clock::Clock,
            {
                /// Puts the two "halves" of a split `Read + Write` back together.
                pub fn reunite<RX_MODE>(self, other: [<Read $Usart>]<CLOCK, RX_MODE>) -> $Usart<CLOCK, RX_MODE>
                where
                    RX_MODE: $crate::port::mode::InputMode,
                {
                    other.reunite(self)
                }
            }

            impl<CLOCK> $crate::hal::serial::Write<u8> for [<Write $Usart>]<CLOCK>
            where
                CLOCK: $crate::clock::Clock,
            {
                type Error = $crate::serial::Error;

                fn write(&mut self, byte: u8) -> $crate::nb::Result<(), Self::Error> {
                    // Call flush to make sure the data-register is empty
                    self.flush()?;

                    self.p.$data.write(|w| unsafe { w.bits(byte) });
                    Ok(())
                }

                fn flush(&mut self) -> $crate::nb::Result<(), Self::Error> {
                    if self.p.$control_a.read().$dre().bit_is_clear() {
                        Err($crate::nb::Error::WouldBlock)
                    } else {
                        Ok(())
                    }
                }
            }

            impl<CLOCK> $crate::ufmt::uWrite for [<Write $Usart>]<CLOCK>
            where
                CLOCK: $crate::clock::Clock,
            {
                type Error = $crate::serial::Error;

                fn write_str(&mut self, s: &str) -> ::core::result::Result<(), Self::Error> {
                    use $crate::prelude::*;

                    for b in s.as_bytes().iter() {
                        $crate::nb::block!(self.write(*b))?;
                    }
                    Ok(())
                }
            }

            impl<CLOCK, RX_MODE> $crate::hal::serial::Read<u8> for [<Read $Usart>]<CLOCK, RX_MODE>
            where
                CLOCK: $crate::clock::Clock,
                RX_MODE: $crate::port::mode::InputMode,
            {
                type Error = $crate::serial::Error;

                fn read(&mut self) -> $crate::nb::Result<u8, Self::Error> {
                    if self.p.$control_a.read().$rxc().bit_is_clear() {
                        return Err($crate::nb::Error::WouldBlock);
                    }

                    Ok(self.p.$data.read().bits())
                }
            }
        }
    };
}
