struct VsOutput {
    float4 pos: SV_Position;
    float2 tex_coord: TEXCOORD;
};

cbuffer Locals {
	float4x4 u_Transform;
};

VsOutput Vertex(float4 pos: a_Pos, float2 tex_coord: a_TexCoord) {
    VsOutput output = {
    	mul(u_Transform, pos),
    	tex_coord,
    };
    return output;
}

Texture2D<float4> t_Color;
SamplerState t_Color_;

float4 Pixel(VsOutput pin): SV_Target {
	return t_Color.Sample(t_Color_, pin.tex_coord);
}
