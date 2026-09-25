// @group(0) @binding(0) var<uniform> camera: Camera;
// @group(0) @binding(1) var<uniform> frustum: Frustum; // 6 planes

// @group(1) @binding(0) var<storage, read> objects: array<InstanceInput>;
// @group(1) @binding(1) var<storage, read_write> visible: array<u32>;
// @group(1) @binding(2) var<storage, read_write> args: DrawSettings;
