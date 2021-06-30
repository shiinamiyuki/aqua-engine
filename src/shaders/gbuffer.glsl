// #define GBUFFER_BINDGROUP
layout(set=GBUFFER_BINDGROUP, binding=0) uniform texture2D t_depth;
// layout(set=1, binding=0, r32f) readonly uniform image2D t_depth;
layout(set=GBUFFER_BINDGROUP, binding=1,rgba32f)  readonly uniform image2D rt0;
layout(set=GBUFFER_BINDGROUP, binding=2,rgba32f)  readonly uniform image2D rt1;
layout(set=GBUFFER_BINDGROUP, binding=3,rgba32f)  readonly uniform image2D rt2;
#ifdef GBUFFER_AOV
layout(set=GBUFFER_BINDGROUP, binding=4,rgba32f)  readonly uniform image2D rt3;
#endif