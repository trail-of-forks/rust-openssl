use bitflags::bitflags;
use foreign_types::ForeignTypeRef;
use libc::{c_int, c_uint, c_ulong, time_t};
use std::net::IpAddr;

use crate::error::ErrorStack;
#[cfg(ossl102)]
use crate::x509::X509PurposeId;
use crate::{cvt, cvt_p};
use openssl_macros::corresponds;

bitflags! {
    /// Flags used to check an `X509` certificate.
    pub struct X509CheckFlags: c_uint {
        const ALWAYS_CHECK_SUBJECT = ffi::X509_CHECK_FLAG_ALWAYS_CHECK_SUBJECT;
        const NO_WILDCARDS = ffi::X509_CHECK_FLAG_NO_WILDCARDS;
        const NO_PARTIAL_WILDCARDS = ffi::X509_CHECK_FLAG_NO_PARTIAL_WILDCARDS;
        const MULTI_LABEL_WILDCARDS = ffi::X509_CHECK_FLAG_MULTI_LABEL_WILDCARDS;
        const SINGLE_LABEL_SUBDOMAINS = ffi::X509_CHECK_FLAG_SINGLE_LABEL_SUBDOMAINS;
        /// Requires OpenSSL 1.1.0 or newer.
        #[cfg(any(ossl110))]
        const NEVER_CHECK_SUBJECT = ffi::X509_CHECK_FLAG_NEVER_CHECK_SUBJECT;

        #[deprecated(since = "0.10.6", note = "renamed to NO_WILDCARDS")]
        const FLAG_NO_WILDCARDS = ffi::X509_CHECK_FLAG_NO_WILDCARDS;
    }
}

bitflags! {
    /// Flags used to verify an `X509` certificate chain.
    pub struct X509VerifyFlags: c_ulong {
        const CB_ISSUER_CHECK = ffi::X509_V_FLAG_CB_ISSUER_CHECK;
        const USE_CHECK_TIME = ffi::X509_V_FLAG_USE_CHECK_TIME;
        const CRL_CHECK = ffi::X509_V_FLAG_CRL_CHECK;
        const CRL_CHECK_ALL = ffi::X509_V_FLAG_CRL_CHECK_ALL;
        const IGNORE_CRITICAL = ffi::X509_V_FLAG_IGNORE_CRITICAL;
        const X509_STRICT = ffi::X509_V_FLAG_X509_STRICT;
        const ALLOW_PROXY_CERTS = ffi::X509_V_FLAG_ALLOW_PROXY_CERTS;
        const POLICY_CHECK = ffi::X509_V_FLAG_POLICY_CHECK;
        const EXPLICIT_POLICY = ffi::X509_V_FLAG_EXPLICIT_POLICY;
        const INHIBIT_ANY = ffi::X509_V_FLAG_INHIBIT_ANY;
        const INHIBIT_MAP = ffi::X509_V_FLAG_INHIBIT_MAP;
        const NOTIFY_POLICY = ffi::X509_V_FLAG_NOTIFY_POLICY;
        const EXTENDED_CRL_SUPPORT = ffi::X509_V_FLAG_EXTENDED_CRL_SUPPORT;
        const USE_DELTAS = ffi::X509_V_FLAG_USE_DELTAS;
        const CHECK_SS_SIGNATURE = ffi::X509_V_FLAG_CHECK_SS_SIGNATURE;
        #[cfg(ossl102)]
        const TRUSTED_FIRST = ffi::X509_V_FLAG_TRUSTED_FIRST;
        #[cfg(ossl102)]
        const SUITEB_128_LOS_ONLY = ffi::X509_V_FLAG_SUITEB_128_LOS_ONLY;
        #[cfg(ossl102)]
        const SUITEB_192_LOS = ffi::X509_V_FLAG_SUITEB_128_LOS;
        #[cfg(ossl102)]
        const SUITEB_128_LOS = ffi::X509_V_FLAG_SUITEB_192_LOS;
        #[cfg(ossl102)]
        const PARTIAL_CHAIN = ffi::X509_V_FLAG_PARTIAL_CHAIN;
        #[cfg(ossl110)]
        const NO_ALT_CHAINS = ffi::X509_V_FLAG_NO_ALT_CHAINS;
        #[cfg(ossl110)]
        const NO_CHECK_TIME = ffi::X509_V_FLAG_NO_CHECK_TIME;
    }
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::X509_VERIFY_PARAM;
    fn drop = ffi::X509_VERIFY_PARAM_free;

    /// Adjust parameters associated with certificate verification.
    pub struct X509VerifyParam;
    /// Reference to `X509VerifyParam`.
    pub struct X509VerifyParamRef;
}

impl X509VerifyParam {
    /// Create an X509VerifyParam
    #[corresponds(X509_VERIFY_PARAM_new)]
    pub fn new() -> Result<X509VerifyParam, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::X509_VERIFY_PARAM_new()).map(X509VerifyParam)
        }
    }
}

impl X509VerifyParamRef {
    /// Set the host flags.
    #[corresponds(X509_VERIFY_PARAM_set_hostflags)]
    pub fn set_hostflags(&mut self, hostflags: X509CheckFlags) {
        unsafe {
            ffi::X509_VERIFY_PARAM_set_hostflags(self.as_ptr(), hostflags.bits);
        }
    }

    /// Set verification flags.
    #[corresponds(X509_VERIFY_PARAM_set_flags)]
    pub fn set_flags(&mut self, flags: X509VerifyFlags) -> Result<(), ErrorStack> {
        unsafe { cvt(ffi::X509_VERIFY_PARAM_set_flags(self.as_ptr(), flags.bits)).map(|_| ()) }
    }

    /// Clear verification flags.
    #[corresponds(X509_VERIFY_PARAM_clear_flags)]
    pub fn clear_flags(&mut self, flags: X509VerifyFlags) -> Result<(), ErrorStack> {
        unsafe {
            cvt(ffi::X509_VERIFY_PARAM_clear_flags(
                self.as_ptr(),
                flags.bits,
            ))
            .map(|_| ())
        }
    }

    /// Gets verification flags.
    #[corresponds(X509_VERIFY_PARAM_get_flags)]
    pub fn flags(&mut self) -> X509VerifyFlags {
        let bits = unsafe { ffi::X509_VERIFY_PARAM_get_flags(self.as_ptr()) };
        X509VerifyFlags { bits }
    }

    /// Set the expected DNS hostname.
    #[corresponds(X509_VERIFY_PARAM_set1_host)]
    pub fn set_host(&mut self, host: &str) -> Result<(), ErrorStack> {
        unsafe {
            // len == 0 means "run strlen" :(
            let raw_host = if host.is_empty() { "\0" } else { host };
            cvt(ffi::X509_VERIFY_PARAM_set1_host(
                self.as_ptr(),
                raw_host.as_ptr() as *const _,
                host.len(),
            ))
            .map(|_| ())
        }
    }

    /// Set the expected IPv4 or IPv6 address.
    #[corresponds(X509_VERIFY_PARAM_set1_ip)]
    pub fn set_ip(&mut self, ip: IpAddr) -> Result<(), ErrorStack> {
        unsafe {
            let mut buf = [0; 16];
            let len = match ip {
                IpAddr::V4(addr) => {
                    buf[..4].copy_from_slice(&addr.octets());
                    4
                }
                IpAddr::V6(addr) => {
                    buf.copy_from_slice(&addr.octets());
                    16
                }
            };
            cvt(ffi::X509_VERIFY_PARAM_set1_ip(
                self.as_ptr(),
                buf.as_ptr() as *const _,
                len,
            ))
            .map(|_| ())
        }
    }

    /// Set the verification time, where time is of type time_t, traditionaly defined as seconds since the epoch
    #[corresponds(X509_VERIFY_PARAM_set_time)]
    pub fn set_time(&mut self, time: time_t) {
        unsafe { ffi::X509_VERIFY_PARAM_set_time(self.as_ptr(), time) }
    }

    /// Set the verification depth
    #[corresponds(X509_VERIFY_PARAM_set_depth)]
    pub fn set_depth(&mut self, depth: c_int) {
        unsafe { ffi::X509_VERIFY_PARAM_set_depth(self.as_ptr(), depth) }
    }

    /// Sets the authentication security level to auth_level
    #[corresponds(X509_VERIFY_PARAM_set_auth_level)]
    #[cfg(ossl110)]
    pub fn set_auth_level(&mut self, lvl: c_int) {
        unsafe { ffi::X509_VERIFY_PARAM_set_auth_level(self.as_ptr(), lvl) }
    }

    /// Gets the current authentication security level
    #[corresponds(X509_VERIFY_PARAM_get_auth_level)]
    #[cfg(ossl110)]
    pub fn auth_level(&self) -> i32 {
        unsafe { ffi::X509_VERIFY_PARAM_get_auth_level(self.as_ptr()) }
    }

    /// Sets the verification purpose
    #[corresponds(X509_VERIFY_PARAM_set_purpose)]
    #[cfg(ossl102)]
    pub fn set_purpose(&mut self, purpose: X509PurposeId) -> Result<(), ErrorStack> {
        unsafe { cvt(ffi::X509_VERIFY_PARAM_set_purpose(self.as_ptr(), purpose.0)).map(|_| ()) }
    }
}
