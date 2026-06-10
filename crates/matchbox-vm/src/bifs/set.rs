use crate::types::{BxNativeFunction, BxVM, BxValue};
use std::collections::HashMap;

pub fn register_set_bifs(bifs: &mut HashMap<String, BxNativeFunction>) {
    bifs.insert("setnew".to_string(), set_new as BxNativeFunction);
    bifs.insert("setof".to_string(), set_of as BxNativeFunction);
    bifs.insert("objecttoset".to_string(), object_to_set as BxNativeFunction);
    bifs.insert("toset".to_string(), to_set as BxNativeFunction);
    bifs.insert("structkeyset".to_string(), struct_key_set as BxNativeFunction);
    bifs.insert("structvalueset".to_string(), struct_value_set as BxNativeFunction);

    bifs.insert("boxsetadd".to_string(), box_set_add as BxNativeFunction);
    bifs.insert("boxsetaddall".to_string(), box_set_add_all as BxNativeFunction);
    bifs.insert("boxsetclear".to_string(), box_set_clear as BxNativeFunction);
    bifs.insert("boxsetremove".to_string(), box_set_remove as BxNativeFunction);
    bifs.insert("boxsetremoveall".to_string(), box_set_remove_all as BxNativeFunction);
    bifs.insert("boxsetretainall".to_string(), box_set_retain_all as BxNativeFunction);

    bifs.insert("boxsetcontains".to_string(), box_set_contains as BxNativeFunction);
    bifs.insert("boxsetcontainsall".to_string(), box_set_contains_all as BxNativeFunction);
    bifs.insert("boxsetisempty".to_string(), box_set_is_empty as BxNativeFunction);
    bifs.insert("boxsetequals".to_string(), box_set_equals as BxNativeFunction);
    bifs.insert("boxsetissubsetof".to_string(), box_set_is_subset_of as BxNativeFunction);
    bifs.insert("boxsetissupersetof".to_string(), box_set_is_superset_of as BxNativeFunction);
    bifs.insert("boxsetisdisjointfrom".to_string(), box_set_is_disjoint_from as BxNativeFunction);
    bifs.insert("boxsetfind".to_string(), box_set_find as BxNativeFunction);

    bifs.insert("boxsetunion".to_string(), box_set_union as BxNativeFunction);
    bifs.insert("boxsetintersection".to_string(), box_set_intersection as BxNativeFunction);
    bifs.insert("boxsetdifference".to_string(), box_set_difference as BxNativeFunction);
    bifs.insert("boxsetsymmetricdifference".to_string(), box_set_symmetric_difference as BxNativeFunction);

    bifs.insert("boxseteach".to_string(), box_set_each as BxNativeFunction);
    bifs.insert("boxsetevery".to_string(), box_set_every as BxNativeFunction);
    bifs.insert("boxsetsome".to_string(), box_set_some as BxNativeFunction);
    bifs.insert("boxsetnone".to_string(), box_set_none as BxNativeFunction);
    bifs.insert("boxsetfilter".to_string(), box_set_filter as BxNativeFunction);
    bifs.insert("boxsetmap".to_string(), box_set_map as BxNativeFunction);
    bifs.insert("boxsetreject".to_string(), box_set_reject as BxNativeFunction);
    bifs.insert("boxsetreduce".to_string(), box_set_reduce as BxNativeFunction);

    bifs.insert("boxsettoarray".to_string(), box_set_to_array as BxNativeFunction);
    bifs.insert("boxsettolist".to_string(), box_set_to_list as BxNativeFunction);
}

fn values_equal(vm: &dyn BxVM, a: BxValue, b: BxValue) -> bool {
    if a.to_bits() == b.to_bits() {
        return true;
    }
    vm.to_string(a) == vm.to_string(b)
}

fn set_contains(vm: &dyn BxVM, set_id: usize, value: BxValue) -> bool {
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        if values_equal(vm, elem, value) {
            return true;
        }
    }
    false
}

fn set_add_unique(vm: &mut dyn BxVM, set_id: usize, value: BxValue) {
    if !set_contains(vm, set_id, value) {
        vm.array_push(set_id, value);
    }
}

fn get_set_id(args: &[BxValue], idx: usize, name: &str) -> Result<usize, String> {
    args.get(idx)
        .and_then(|v| v.as_gc_id())
        .ok_or_else(|| format!("{}() expects a set (array) as argument {}", name, idx + 1))
}

fn set_new(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.array_new()))
}

fn set_of(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.array_new();
    for arg in args {
        set_add_unique(vm, id, *arg);
    }
    Ok(BxValue::new_ptr(id))
}

fn object_to_set(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("objectToSet() expects 1 argument".to_string());
    }
    let src_id = get_set_id(args, 0, "objectToSet")?;
    let result_id = vm.array_new();
    let len = vm.array_len(src_id);
    for i in 0..len {
        let elem = vm.array_get(src_id, i);
        set_add_unique(vm, result_id, elem);
    }
    Ok(BxValue::new_ptr(result_id))
}

fn to_set(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("toSet() expects 1 argument".to_string());
    }
    if let Some(src_id) = args[0].as_gc_id() {
        let result_id = vm.array_new();
        let len = vm.array_len(src_id);
        for i in 0..len {
            let elem = vm.array_get(src_id, i);
            set_add_unique(vm, result_id, elem);
        }
        Ok(BxValue::new_ptr(result_id))
    } else {
        let result_id = vm.array_new();
        set_add_unique(vm, result_id, args[0]);
        Ok(BxValue::new_ptr(result_id))
    }
}

fn struct_key_set(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("structKeySet() expects 1 argument".to_string());
    }
    let struct_id = args[0]
        .as_gc_id()
        .ok_or("structKeySet() expects a struct argument".to_string())?;
    let keys = vm.struct_key_array(struct_id);
    let result_id = vm.array_new();
    for key in keys {
        let s_id = vm.string_new(key);
        set_add_unique(vm, result_id, BxValue::new_ptr(s_id));
    }
    Ok(BxValue::new_ptr(result_id))
}

fn struct_value_set(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("structValueSet() expects 1 argument".to_string());
    }
    let struct_id = args[0]
        .as_gc_id()
        .ok_or("structValueSet() expects a struct argument".to_string())?;
    let keys = vm.struct_key_array(struct_id);
    let result_id = vm.array_new();
    for key in keys {
        let val = vm.struct_get(struct_id, &key);
        set_add_unique(vm, result_id, val);
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_add(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetAdd() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetAdd")?;
    set_add_unique(vm, set_id, args[1]);
    Ok(args[0])
}

fn box_set_add_all(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetAddAll() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetAddAll")?;
    let coll_id = get_set_id(args, 1, "boxSetAddAll")?;
    let len = vm.array_len(coll_id);
    for i in 0..len {
        let elem = vm.array_get(coll_id, i);
        set_add_unique(vm, set_id, elem);
    }
    Ok(args[0])
}

fn box_set_clear(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("boxSetClear() expects 1 argument".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetClear")?;
    vm.array_clear(set_id)?;
    Ok(BxValue::new_null())
}

fn box_set_remove(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetRemove() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetRemove")?;
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        if values_equal(vm, elem, args[1]) {
            vm.array_delete_at(set_id, i)?;
            break;
        }
    }
    Ok(args[0])
}

fn box_set_remove_all(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetRemoveAll() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetRemoveAll")?;
    let coll_id = get_set_id(args, 1, "boxSetRemoveAll")?;
    let coll_len = vm.array_len(coll_id);
    let mut to_remove: Vec<BxValue> = Vec::new();
    for i in 0..coll_len {
        to_remove.push(vm.array_get(coll_id, i));
    }
    for val in &to_remove {
        let set_len = vm.array_len(set_id);
        for j in 0..set_len {
            let elem = vm.array_get(set_id, j);
            if values_equal(vm, elem, *val) {
                vm.array_delete_at(set_id, j)?;
                break;
            }
        }
    }
    Ok(args[0])
}

fn box_set_retain_all(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetRetainAll() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetRetainAll")?;
    let coll_id = get_set_id(args, 1, "boxSetRetainAll")?;
    let mut i = 0;
    while i < vm.array_len(set_id) {
        let elem = vm.array_get(set_id, i);
        if !set_contains(vm, coll_id, elem) {
            vm.array_delete_at(set_id, i)?;
        } else {
            i += 1;
        }
    }
    Ok(args[0])
}

fn box_set_contains(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetContains() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetContains")?;
    Ok(BxValue::new_bool(set_contains(vm, set_id, args[1])))
}

fn box_set_contains_all(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetContainsAll() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetContainsAll")?;
    let coll_id = get_set_id(args, 1, "boxSetContainsAll")?;
    let len = vm.array_len(coll_id);
    for i in 0..len {
        let elem = vm.array_get(coll_id, i);
        if !set_contains(vm, set_id, elem) {
            return Ok(BxValue::new_bool(false));
        }
    }
    Ok(BxValue::new_bool(true))
}

fn box_set_is_empty(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("boxSetIsEmpty() expects 1 argument".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetIsEmpty")?;
    Ok(BxValue::new_bool(vm.array_len(set_id) == 0))
}

fn box_set_equals(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetEquals() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetEquals")?;
    let set2_id = get_set_id(args, 1, "boxSetEquals")?;
    let len1 = vm.array_len(set1_id);
    let len2 = vm.array_len(set2_id);
    if len1 != len2 {
        return Ok(BxValue::new_bool(false));
    }
    for i in 0..len1 {
        let elem = vm.array_get(set1_id, i);
        if !set_contains(vm, set2_id, elem) {
            return Ok(BxValue::new_bool(false));
        }
    }
    Ok(BxValue::new_bool(true))
}

fn box_set_is_subset_of(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetIsSubsetOf() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetIsSubsetOf")?;
    let set2_id = get_set_id(args, 1, "boxSetIsSubsetOf")?;
    let len = vm.array_len(set1_id);
    for i in 0..len {
        let elem = vm.array_get(set1_id, i);
        if !set_contains(vm, set2_id, elem) {
            return Ok(BxValue::new_bool(false));
        }
    }
    Ok(BxValue::new_bool(true))
}

fn box_set_is_superset_of(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetIsSupersetOf() expects 2 arguments".to_string());
    }
    let reversed = [args[1], args[0]];
    box_set_is_subset_of(vm, &reversed)
}

fn box_set_is_disjoint_from(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetIsDisjointFrom() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetIsDisjointFrom")?;
    let set2_id = get_set_id(args, 1, "boxSetIsDisjointFrom")?;
    let len = vm.array_len(set1_id);
    for i in 0..len {
        let elem = vm.array_get(set1_id, i);
        if set_contains(vm, set2_id, elem) {
            return Ok(BxValue::new_bool(false));
        }
    }
    Ok(BxValue::new_bool(true))
}

fn box_set_find(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetFind() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetFind")?;
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        if values_equal(vm, elem, args[1]) {
            return Ok(elem);
        }
    }
    Ok(BxValue::new_null())
}

fn box_set_union(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetUnion() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetUnion")?;
    let set2_id = get_set_id(args, 1, "boxSetUnion")?;
    let result_id = vm.array_new();
    let len1 = vm.array_len(set1_id);
    for i in 0..len1 {
        let elem = vm.array_get(set1_id, i);
        set_add_unique(vm, result_id, elem);
    }
    let len2 = vm.array_len(set2_id);
    for i in 0..len2 {
        let elem = vm.array_get(set2_id, i);
        set_add_unique(vm, result_id, elem);
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_intersection(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetIntersection() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetIntersection")?;
    let set2_id = get_set_id(args, 1, "boxSetIntersection")?;
    let result_id = vm.array_new();
    let len = vm.array_len(set1_id);
    for i in 0..len {
        let elem = vm.array_get(set1_id, i);
        if set_contains(vm, set2_id, elem) {
            set_add_unique(vm, result_id, elem);
        }
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_difference(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetDifference() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetDifference")?;
    let set2_id = get_set_id(args, 1, "boxSetDifference")?;
    let result_id = vm.array_new();
    let len = vm.array_len(set1_id);
    for i in 0..len {
        let elem = vm.array_get(set1_id, i);
        if !set_contains(vm, set2_id, elem) {
            vm.array_push(result_id, elem);
        }
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_symmetric_difference(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetSymmetricDifference() expects 2 arguments".to_string());
    }
    let set1_id = get_set_id(args, 0, "boxSetSymmetricDifference")?;
    let set2_id = get_set_id(args, 1, "boxSetSymmetricDifference")?;
    let result_id = vm.array_new();
    let len1 = vm.array_len(set1_id);
    for i in 0..len1 {
        let elem = vm.array_get(set1_id, i);
        if !set_contains(vm, set2_id, elem) {
            vm.array_push(result_id, elem);
        }
    }
    let len2 = vm.array_len(set2_id);
    for i in 0..len2 {
        let elem = vm.array_get(set2_id, i);
        if !set_contains(vm, set1_id, elem) {
            vm.array_push(result_id, elem);
        }
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_each(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetEach() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetEach")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
    }
    Ok(BxValue::new_null())
}

fn box_set_every(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetEvery() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetEvery")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        let r = vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
        if !r.as_bool() {
            return Ok(BxValue::new_bool(false));
        }
    }
    Ok(BxValue::new_bool(true))
}

fn box_set_some(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetSome() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetSome")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        let r = vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
        if r.as_bool() {
            return Ok(BxValue::new_bool(true));
        }
    }
    Ok(BxValue::new_bool(false))
}

fn box_set_none(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetNone() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetNone")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        let r = vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
        if r.as_bool() {
            return Ok(BxValue::new_bool(false));
        }
    }
    Ok(BxValue::new_bool(true))
}

fn box_set_filter(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetFilter() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetFilter")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let result_id = vm.array_new();
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        let r = vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
        if r.as_bool() {
            vm.array_push(result_id, elem);
        }
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_map(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetMap() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetMap")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let result_id = vm.array_new();
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        let mapped = vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
        set_add_unique(vm, result_id, mapped);
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_reject(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("boxSetReject() expects 2 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetReject")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let result_id = vm.array_new();
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        let r = vm.call_function_by_value(&args[1], vec![elem], chunk.clone())?;
        if !r.as_bool() {
            vm.array_push(result_id, elem);
        }
    }
    Ok(BxValue::new_ptr(result_id))
}

fn box_set_reduce(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 3 {
        return Err("boxSetReduce() expects 3 arguments".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetReduce")?;
    let chunk = vm.current_chunk().ok_or("No current chunk")?;
    let mut acc = args[2];
    let len = vm.array_len(set_id);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        acc = vm.call_function_by_value(&args[1], vec![acc, elem], chunk.clone())?;
    }
    Ok(acc)
}

fn box_set_to_array(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("boxSetToArray() expects 1 argument".to_string());
    }
    let _set_id = get_set_id(args, 0, "boxSetToArray")?;
    Ok(args[0])
}

fn box_set_to_list(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("boxSetToList() expects at least 1 argument".to_string());
    }
    let set_id = get_set_id(args, 0, "boxSetToList")?;
    let delim = if args.len() > 1 {
        vm.to_string(args[1])
    } else {
        ",".to_string()
    };
    let len = vm.array_len(set_id);
    let mut parts: Vec<String> = Vec::with_capacity(len);
    for i in 0..len {
        let elem = vm.array_get(set_id, i);
        parts.push(vm.to_string(elem));
    }
    let result = parts.join(&delim);
    let s_id = vm.string_new(result);
    Ok(BxValue::new_ptr(s_id))
}
