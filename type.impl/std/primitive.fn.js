(function() {
    var type_impls = Object.fromEntries([["wayland_sys",[]],["x11rb",[]],["x11rb_protocol",[]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[18,13,22]}