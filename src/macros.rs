macro_rules! init_features {
    (
        @MACRO: $macro:ident;
        $(@FEATURE: $feature_mode:ident: $feature:tt;)*
    ) => (
        macro_rules! feature_detected {
            $(
                ($feature_mode) => {
                    $macro!($feature)
                };
            )*
            ($unknown_feature_mode:tt) => {
                false
            };
        }
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
init_features! {
    @MACRO: is_x86_feature_detected;
    @FEATURE: 
        normal: "sse2";
    @FEATURE: 
        extra: "avx2";
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
init_features! {
    @MACRO: is_aarch64_feature_detected;
    @FEATURE: 
        normal: "neon";
}