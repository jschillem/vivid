struct Camera {
  float4x4 view_proj;
}

struct VertexInput {
  float3 position;
  float3 normal;
}

struct InstanceInput {
  float3 offset;
}

struct VertexOutput {
  float4 position : SV_Position;
  float3 normal;
}

ConstantBuffer<Camera> camera;

[shader("vertex")]
VertexOutput vs_main(VertexInput vert, InstanceInput inst) {
  let world = vert.position + inst.offset;
  VertexOutput output;
  output.position = mul(camera.view_proj, float4(world, 1.0));
  return output;
}

[shader("fragment")]
float4 fs_main(VertexOutput input) : SV_Target {
  return float4(0.9, 0.35, 0.2, 1.0);
}

