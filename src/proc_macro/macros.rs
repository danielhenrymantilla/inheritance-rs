#[cfg(not(feature = "verbose-expansions"))]
macro_rules! debug {(
    $($tt:tt)*
) => (
)}
#[cfg(feature = "verbose-expansions")]
macro_rules! debug {(
    $($tt:tt)*
) => (
    eprintln!($($tt)*)
)}

macro_rules! error {
    (
        $span:expr, $msg:expr $(,)?
    ) => ({
        let span: Span = $span;
        let msg = LitStr::new(&$msg, span);
        return TokenStream::from(quote_spanned! {span =>
            compile_error!(#msg);
        });
    });

    (
        $msg:expr $(,)?
    ) => (
        error! { Span::call_site()=>
            $msg
        }
    );
}

macro_rules! parse_error {
    (
        $span:expr, $msg:expr $(,)?
    ) => (
        return Err(Error::new($span, $msg));
    );

    (
        $msg:expr $(,)?
    ) => (
        parse_error! { Span::call_site()=>
            $msg
        }
    );
}

macro_rules! set_output {
    (@with_dollar![$dol:tt]
        $render:ident => $ret:ident
    ) => (
        #[allow(unused_mut)]
        let mut $ret = TokenStream::new();

        macro_rules! $render {
            ($dol span:expr =>
                $dol($dol tt:tt)*
            ) => ({
                let span: Span = $dol span;
                $ret.extend(TokenStream::from(quote_spanned! { span=>
                    $dol($dol tt)*
                }));
            });

            (
                $dol($dol tt:tt)*
            ) => ($render! { Span::call_site() =>
                $dol($dol tt)*
            });
        }

    );

    (
        $($tt:tt)*
    ) => (set_output!(@with_dollar![$]
        $($tt)*
    ));
}
