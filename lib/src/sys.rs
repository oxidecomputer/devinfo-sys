// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2022 Oxide Computer Company

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::ffi::{c_void, CStr};
use std::io::{Error, Result};
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_ulong};
use std::ptr::{null, null_mut};
use std::slice;

use crate::{DeviceInfo, DiPropType, DiPropValue};

const DIIOC: u32 = 0xdf << 8;
const DINFOSUBTREE: u32 = DIIOC | 0x01; /* include subtree */
const DINFOMINOR: u32 = DIIOC | 0x02; /* include minor data */
const DINFOPROP: u32 = DIIOC | 0x04; /* include properties */
const DINFOPATH: u32 = DIIOC | 0x08; /* include i/o pathing information */

const DI_WALK_CONTINUE: c_int = 0;
const DI_WALK_PRUNESIB: c_int = -1;
const DI_WALK_PRUNECHILD: c_int = -2;
const DI_WALK_TERMINATE: c_int = -3;

const DI_NODE_NIL: *const c_void = null();
const DI_MINOR_NIL: *const c_void = null();
const DI_PATH_NIL: *const c_void = null();
const DI_LINK_NIL: *const c_void = null();
const DI_LNODE_NIL: *const c_void = null();
const DI_PROP_NIL: *const c_void = null();
const DI_PROM_PROP_NIL: *const c_void = null();
const DI_PROM_HANDLE_NIL: *const c_void = null();
const DI_HP_NIL: *const c_void = null();

const DI_WALK_CLDFIRST: c_uint = 0;
const DI_WALK_SIBFIRST: c_uint = 1;
const DI_WALK_LINKGEN: c_uint = 2;

const OPROMMAXPARAM: c_uint = 32768;

type di_off_t = u32;

#[repr(C)]
enum ddi_node_state_t {
    DS_INVAL = -1,
    DS_PROTO = 0,
    DS_LINKED,      /* in orphan list */
    DS_BOUND,       /* in per-driver list */
    DS_INITIALIZED, /* bus address assigned */
    DS_PROBED,      /* device known to exist */
    DS_ATTACHED,    /* don't use, see NOTE above: driver attached */
    DS_READY,       /* don't use, see NOTE above: post attach complete */
}

#[repr(C)]
enum ddi_node_class_t {
    DDI_NC_PROM = 0,
    DDI_NC_PSEUDO,
}

#[repr(C)]
enum ddi_minor_type {
    DDM_MINOR = 0,
    DDM_ALIAS,
    DDM_DEFAULT,
    DDM_INTERNAL_PATH,
}

#[repr(C)]
struct di_node {
    /*
     * offset to di_node structures
     */
    _self: di_off_t,   /* make it self addressable */
    parent: di_off_t,  /* offset of parent node */
    child: di_off_t,   /* offset of child node */
    sibling: di_off_t, /* offset of sibling */
    next: di_off_t,    /* next node on per-instance list */

    /*
     * offset to char strings of current node
     */
    node_name: di_off_t,    /* offset of device node name */
    address: di_off_t,      /* offset of address part of name */
    bind_name: di_off_t,    /* offset of binding name */
    compat_names: di_off_t, /* offset of compatible names */

    /*
     * offset to property lists, private data, etc.
     */
    minor_data: di_off_t,
    drv_prop: di_off_t,
    sys_prop: di_off_t,
    glob_prop: di_off_t,
    hw_prop: di_off_t,
    parent_data: di_off_t,
    driver_data: di_off_t,
    multipath_client: di_off_t,
    multipath_phci: di_off_t,
    devid: di_off_t,   /* registered device id */
    pm_info: di_off_t, /* RESERVED FOR FUTURE USE */

    /*
     * misc values
     */
    compat_length: c_int, /* size of compatible name list */
    drv_major: c_int,     /* for indexing into devnames array */

    /*
     * value attributes of current node
     */
    instance: c_int,              /* instance number */
    nodeid: c_int,                /* node id */
    node_class: ddi_node_class_t, /* node class */
    attributes: c_int,            /* node attributes */
    state: c_uint,                /* hotplugging device state */
    node_state: ddi_node_state_t, /* devinfo state */

    lnodes: di_off_t, /* lnodes associated with this di_node */
    tgt_links: di_off_t,
    src_links: di_off_t,

    di_pad1: u32, /* 4 byte padding for 32bit x86 app. */
    user_private_data: u64,

    /*
     * offset to link vhci/phci nodes.
     */
    next_vhci: di_off_t,
    top_phci: di_off_t,
    next_phci: di_off_t,
    multipath_component: u32, /* stores MDI_COMPONENT_* value. */

    /*
     * devi_flags field
     */
    flags: u32,
    di_pad2: u32, /* 4 byte padding for 32bit x86 app. */

    /*
     * offset to hotplug nodes.
     */
    hp_data: di_off_t,
}

#[repr(C)]
struct di_prop {
    _self: di_off_t, /* make it self addressable */
    next: di_off_t,
    prop_name: di_off_t, /* Property name */
    prop_data: di_off_t, /* property data */
    dev_major: major_t,  /* dev_t can be 64 bit */
    dev_minor: minor_t,
    prop_flags: c_int, /* mark prop value types & more */
    prop_len: c_int,   /* prop len in bytes (boolean if 0) */
    prop_list: c_int,  /* which list (DI_PROP_SYS_LIST), etc */
}

#[repr(C)]
struct di_prom_handle {
    lock: mutex_t,           /* synchronize access to openprom fd */
    fd: c_int,               /* /dev/openprom file descriptor */
    list: *mut di_prom_prop, /* linked list of prop */
    oppbuf: OppBuf,
}

#[repr(C)]
struct di_prom_prop {
    name: *mut c_char,
    len: c_int,
    data: *mut c_uchar,
    next: *mut di_prom_prop, /* form a linked list */
}

#[repr(C)]
union OppBuf {
    buf: [c_char; OPROMMAXPARAM as usize],
    opp: openpromio,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct openpromio {
    oprom_size: c_uint, /* real size of following array */
    opio_u: openpromio_opio,
}

#[derive(Copy, Clone)]
#[repr(C)]
union openpromio_opio {
    b: [c_char; 1], /* For property names and values */
    /* NB: Adjacent, Null terminated */
    i: c_int,
}

#[repr(C)]
struct di_minor {
    _self: di_off_t,       /* make it self addressable */
    next: di_off_t,        /* next one in the chain */
    name: di_off_t,        /* name of node */
    node_type: di_off_t,   /* block, byte, serial, network */
    _type: ddi_minor_type, /* data type */
    dev_major: major_t,    /* dev_t can be 64-bit */
    dev_minor: minor_t,
    spec_type: c_int, /* block or char */
    mdclass: c_uint,  /* no longer used, may be removed */
    node: di_off_t,   /* address of di_node */
    user_private_data: u64,
}

#[repr(C)]
struct lwp_mutex_t {
    flags: lwp_mutex_flags,
    lock: lwp_mutex_lock,
    data: u64,
}

#[repr(C)]
struct lwp_mutex_flags {
    flag1: u16,
    flag2: u8,
    ceiling: u8,
    mbcp_type_un: lwp_mutex_flags_mbcp_type,
    magic: u16,
}

#[repr(C)]
union lwp_mutex_flags_mbcp_type {
    bcptype: u16,
    mtype_rcount: lwp_mutex_flags_mbcp_type_mtype_rcount,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct lwp_mutex_flags_mbcp_type_mtype_rcount {
    count_type1: u8,
    count_type2: u8,
}

#[repr(C)]
union lwp_mutex_lock {
    lock64: lwp_mutex_lock_lock64,
    lock32: lwp_mutex_lock_lock32,
    owner64: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct lwp_mutex_lock_lock64 {
    pad: [u8; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
struct lwp_mutex_lock_lock32 {
    ownerpid: u32,
    lockword: u32,
}

type di_node_t = *mut di_node;
type di_minor_t = *mut di_minor;
type di_prop_t = *mut di_prop;
type di_prom_handle_t = *mut di_prom_handle;
type di_prom_prop_t = *mut di_prom_prop;
type major_t = c_ulong;
type minor_t = c_ulong;
type mutex_t = lwp_mutex_t;

extern "C" {
    fn di_init(phys_path: *const c_char, flags: c_uint) -> di_node_t;
    fn di_walk_node(
        root: di_node_t,
        flag: c_uint,
        arg: *mut c_void,
        node_callback: extern "C" fn(di_node_t, *mut c_void) -> c_int,
    ) -> c_int;
    fn di_fini(root: di_node_t);
    fn di_node_name(node: di_node_t) -> *const c_char;
    fn di_minor_next(node: di_node_t, minor: di_minor_t) -> di_minor_t;
    fn di_instance(node: di_node_t) -> c_int;
    fn di_devfs_path(node: di_node_t) -> *const c_char;
    fn di_drv_first_node(drv_name: *const c_char, root: di_node_t)
        -> di_node_t;
    fn di_drv_next_node(node: di_node_t) -> di_node_t;

    fn di_prop_next(node: di_node_t, prop: di_prop_t) -> di_prop_t;
    fn di_prop_name(prop: di_prop_t) -> *const c_char;
    fn di_prop_type(prop: di_prop_t) -> c_int;
    fn di_prop_bytes(prop: di_prop_t, prop_data: *mut *mut c_uchar) -> c_int;
    fn di_prop_ints(prop: di_prop_t, prop_data: *mut *mut c_int) -> c_int;
    fn di_prop_int64(prop: di_prop_t, prop_data: *mut *mut i64) -> c_int;
    fn di_prop_strings(prop: di_prop_t, prop_data: *mut *mut c_char) -> c_int;

    fn di_prom_init() -> di_prom_handle_t;
    fn di_prom_prop_next(
        ph: di_prom_handle_t,
        node: di_node_t,
        prom_prop: di_prom_prop_t,
    ) -> di_prom_prop_t;
    fn di_prom_fini(ph: di_prom_handle_t);
    fn di_prom_prop_name(prom_prop: di_prom_prop_t) -> *const c_char;
    fn di_prom_prop_data(
        prom_prop: di_prom_prop_t,
        prom_prop_data: *mut *mut c_uchar,
    ) -> c_int;
}

struct Context {
    info: BTreeMap<String, DeviceInfo>,
    fetch_prom: bool,
}

pub fn get_devices(fetch_prom: bool) -> Result<BTreeMap<String, DeviceInfo>> {
    let path = std::ffi::CString::new("/").unwrap();
    let root_node = unsafe {
        di_init(
            path.as_c_str().as_ptr() as *const c_char,
            DINFOSUBTREE | DINFOPROP,
        )
    };
    if root_node.is_null() {
        return Err(Error::last_os_error());
    }

    let mut ctx = Context {
        info: BTreeMap::new(),
        fetch_prom,
    };

    unsafe {
        di_walk_node(
            root_node,
            DI_WALK_CLDFIRST,
            &mut ctx as *mut Context as *mut c_void,
            node_info,
        );
        di_fini(root_node);
    };

    Ok(ctx.info)
}

fn print_err(msg: String) {
    let err = std::io::Error::last_os_error();
    println!("{}: {}", msg, err);
}

extern "C" fn node_info(node: di_node_t, arg: *mut c_void) -> c_int {
    let ctx = unsafe { &mut *(arg as *mut Context) };

    let cs = unsafe { CStr::from_ptr(di_node_name(node)) };
    let node_name = cs.to_str().unwrap();

    let mut info = DeviceInfo::new();

    let mut prop: di_prop_t = null_mut();
    loop {
        prop = unsafe { di_prop_next(node, prop) };
        if prop.is_null() {
            break;
        }

        let cs = unsafe { CStr::from_ptr(di_prop_name(prop)) };
        let prop_name = cs.to_str().unwrap();

        let prop_type = unsafe { di_prop_type(prop) };
        match DiPropType::try_from(prop_type) {
            Ok(t) => match t {
                DiPropType::Boolean => {
                    //existence implies true
                    info.props.insert(
                        prop_name.to_string(),
                        DiPropValue::Boolean(true),
                    );
                }
                DiPropType::Int => {
                    let mut data: *mut i32 = null_mut();
                    let count = unsafe { di_prop_ints(prop, &mut data) };
                    if count < 0 {
                        print_err(format!("{} failed to get ints", prop_name));
                        continue;
                    }
                    let values: &[i32] = unsafe {
                        slice::from_raw_parts_mut(data, count as usize)
                    };

                    info.props.insert(
                        prop_name.to_string(),
                        DiPropValue::Ints(Vec::from(values)),
                    );
                }
                DiPropType::Int64 => {
                    let mut data: *mut i64 = null_mut();
                    let count = unsafe { di_prop_int64(prop, &mut data) };
                    if count < 0 {
                        print_err(format!(
                            "{} failed to get int64s",
                            prop_name
                        ));
                        continue;
                    }
                    let values: &[i64] = unsafe {
                        slice::from_raw_parts_mut(data, count as usize)
                    };

                    info.props.insert(
                        prop_name.to_string(),
                        DiPropValue::Int64s(Vec::from(values)),
                    );
                }
                DiPropType::String => {
                    let mut data: *mut c_char = null_mut();
                    let count = unsafe { di_prop_strings(prop, &mut data) };
                    if count < 0 {
                        print_err(format!(
                            "{} failed to get strings",
                            prop_name
                        ));
                        continue;
                    }

                    let bytes: &mut [u8] = unsafe {
                        slice::from_raw_parts_mut(
                            data as *mut u8,
                            count as usize,
                        )
                    };

                    let concat_str =
                        unsafe { std::str::from_utf8_unchecked_mut(bytes) };
                    let values: Vec<&str> =
                        concat_str.split_terminator('\0').collect();

                    let mut vals = Vec::new();
                    for x in &values {
                        vals.push(x.to_string());
                    }
                    info.props.insert(
                        prop_name.to_string(),
                        DiPropValue::Strings(vals),
                    );
                }
                _ => {}
            },
            Err(_) => continue,
        };
    }

    if ctx.fetch_prom {
        let ph = unsafe { di_prom_init() };
        if ph.is_null() {
            print_err("di_promi_init".to_string());
            return DI_WALK_CONTINUE;
        }

        let mut prom_prop: di_prom_prop_t = null_mut();
        loop {
            prom_prop = unsafe { di_prom_prop_next(ph, node, prom_prop) };
            if prom_prop.is_null() {
                break;
            }

            let cs = unsafe { CStr::from_ptr(di_prom_prop_name(prom_prop)) };
            let prop_name = cs.to_str().unwrap();

            let mut data: *mut c_uchar = null_mut();
            let len = unsafe { di_prom_prop_data(prom_prop, &mut data) };
            if len < 0 {
                print_err(format!("{} get bytes", prop_name));
                continue;
            }
            let bytes =
                unsafe { slice::from_raw_parts_mut(data, len as usize) };
            info.prom_props
                .insert(prop_name.to_string(), Vec::from(bytes));
        }
        unsafe { di_prom_fini(ph) };
    }

    ctx.info.insert(node_name.to_string(), info);

    DI_WALK_CONTINUE
}
