// Source: Kajiya (https://github.com/EmbarkStudios/kajiya)
// Copyright (c) 2019 Embark Studios
// License: Apache-2.0

struct ShadowPayload {
    bool is_shadowed;
};

[shader("miss")]
void main(inout ShadowPayload payload : SV_RayPayload) {
    payload.is_shadowed = false;
}
