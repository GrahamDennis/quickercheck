macro_rules! macro_rec {
    ($name: ident ($first: ident $(, $ident: ident)*)) => {
        $name! { $first $(, $ident)* }
        macro_rec! { $name ( $($ident),* ) }
    };
    ($name: ident ()) => {
        $name! {}
    }
}

macro_rules! macro_tuples_impl {
    ($name: ident) => { macro_rec! {$name (
        TuplesImplA, TuplesImplB, TuplesImplC, TuplesImplD, TuplesImplE, TuplesImplF, TuplesImplG,
        TuplesImplH, TuplesImplI, TuplesImplJ, TuplesImplK, TuplesImplL
    )}}
}
