// Source: Kajiya (https://github.com/EmbarkStudios/kajiya)
// Copyright (c) 2019 Embark Studios
// License: Apache-2.0

#ifndef WRC_BINDINGS_HLSL
#define WRC_BINDINGS_HLSL

#define DEFINE_WRC_BINDINGS(b0) \
    [[vk::binding(b0)]] Texture2D<float4> wrc_radiance_atlas_tex;

#endif  // WRC_BINDINGS_HLSL