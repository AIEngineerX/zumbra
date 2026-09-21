"""The eclipse opener, built from the mark's own geometry in Blender (headless).

  blender -b -P brand/tools/opener.py -- <out_dir>

3 seconds at 24 fps, 1920x1080. A void disc transits across the gold corona (frames 1-44), then
the Z is cut out of the disc as negative space (frames 46-66) and the mark holds. Colours are the
brand hex values under the Standard view transform, so they land exactly. Nothing is generated;
every shape is the SVG's numbers."""
import bpy, math, random, sys
from mathutils import Vector

out_dir = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else "//opener"
FPS, FRAMES = 24, 72

def srgb(hexstr):
    v = [int(hexstr[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    lin = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in v]
    return (*lin, 1.0)

GOLD, VOID = srgb("F2C14E"), srgb("0B0D12")

# ---- clean scene --------------------------------------------------------------------------
bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.render.resolution_x, scene.render.resolution_y, scene.render.resolution_percentage = 1920, 1080, 100
scene.render.fps = FPS
scene.frame_start, scene.frame_end = 1, FRAMES
scene.render.image_settings.file_format = "PNG"
scene.render.filepath = out_dir.rstrip("/\\") + "/f"
scene.view_settings.view_transform = "Standard"
try:
    scene.render.engine = "BLENDER_EEVEE_NEXT"
except TypeError:
    scene.render.engine = "BLENDER_EEVEE"

world = bpy.data.worlds.new("void"); scene.world = world; world.use_nodes = True
bg = next(n for n in world.node_tree.nodes if n.type == "BACKGROUND")
bg.inputs["Color"].default_value = VOID; bg.inputs["Strength"].default_value = 1.0

def emissive(name, color, strength):
    m = bpy.data.materials.new(name); m.use_nodes = True
    nt = m.node_tree
    for n in list(nt.nodes): nt.nodes.remove(n)
    out = nt.nodes.new("ShaderNodeOutputMaterial"); em = nt.nodes.new("ShaderNodeEmission")
    em.inputs["Color"].default_value = color; em.inputs["Strength"].default_value = strength
    nt.links.new(em.outputs["Emission"], out.inputs["Surface"])
    return m

mat_gold = emissive("corona", GOLD, 1.0)   # strength 1 under Standard = the exact hex
mat_void = emissive("umbra", VOID, 1.0)      # the "shadow": exactly the ground colour, no light
mat_star = emissive("star", srgb("F4F1EA"), 2.0)

# ---- the corona disc: SVG circle r=200 of 512 -> radius 2.0 in a 5.12 unit frame ---------
S = 1 / 100.0
bpy.ops.mesh.primitive_cylinder_add(vertices=256, radius=200 * S, depth=0.02, location=(0, 0, 0))
disc = bpy.context.object; disc.name = "corona"; disc.data.materials.append(mat_gold)

# ---- the Z: stroke-width 52 path M150 170 H362 L150 342 H362, as three bars in front -------
def svg_xy(x, y):
    return ((x - 256) * S, (256 - y) * S)

def bar(name, p0, p1, width, z):
    (x0, y0), (x1, y1) = svg_xy(*p0), svg_xy(*p1)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    length = math.hypot(x1 - x0, y1 - y0)
    bpy.ops.mesh.primitive_plane_add(size=1, location=(cx, cy, z))
    o = bpy.context.object; o.name = name
    o.scale = (length + width * S * 0.0, width * S, 1)   # ends are squared by the overlap below
    o.rotation_euler = (0, 0, math.atan2(y1 - y0, x1 - x0))
    o.data.materials.append(mat_void)
    return o

W = 52
z_parts = [
    bar("z_top", (150 - 26, 170), (362 + 26, 170), W, 0.05),
    bar("z_diag", (362 + 6, 170 - 8), (150 - 6, 342 + 8), W, 0.05),
    bar("z_bot", (150 - 26, 342), (362 + 26, 342), W, 0.05),
]
# keep the Z inside the disc: the SVG mask only cuts where the circle is, so clip the bars
bpy.ops.mesh.primitive_cylinder_add(vertices=256, radius=200 * S, depth=0.5, location=(0, 0, 0.05))
clip = bpy.context.object; clip.name = "clip"; clip.hide_render = True; clip.hide_viewport = True
for o in z_parts:
    mod = o.modifiers.new("clip", "BOOLEAN"); mod.operation = "INTERSECT"; mod.object = clip; mod.solver = "EXACT"
z_group = bpy.data.objects.new("z", None); scene.collection.objects.link(z_group)
for o in z_parts:
    o.parent = z_group

# ---- corona halo: a soft radial glow behind the disc, geometry not post-process ----------
halo_m = bpy.data.materials.new("halo"); halo_m.use_nodes = True
nt = halo_m.node_tree
for n in list(nt.nodes): nt.nodes.remove(n)
out = nt.nodes.new("ShaderNodeOutputMaterial")
mix = nt.nodes.new("ShaderNodeMixShader")
transp = nt.nodes.new("ShaderNodeBsdfTransparent")
em_h = nt.nodes.new("ShaderNodeEmission"); em_h.inputs["Color"].default_value = GOLD; em_h.inputs["Strength"].default_value = 0.6
coord = nt.nodes.new("ShaderNodeTexCoord"); grad = nt.nodes.new("ShaderNodeTexGradient"); grad.gradient_type = "SPHERICAL"
ramp = nt.nodes.new("ShaderNodeValToRGB")
# object space: plane half-size 3.2 -> gradient 1 at centre, 0 at r=3.2. Disc edge r=2.0 -> 0.375.
ramp.color_ramp.elements[0].position = 0.0;  ramp.color_ramp.elements[0].color = (0, 0, 0, 1)
ramp.color_ramp.elements[1].position = 0.38; ramp.color_ramp.elements[1].color = (1, 1, 1, 1)
ramp.color_ramp.interpolation = "EASE"
nt.links.new(coord.outputs["Object"], grad.inputs["Vector"])
nt.links.new(grad.outputs["Fac"], ramp.inputs["Fac"])
nt.links.new(ramp.outputs["Color"], mix.inputs["Fac"])
nt.links.new(transp.outputs["BSDF"], mix.inputs[1])
nt.links.new(em_h.outputs["Emission"], mix.inputs[2])
nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])
try: halo_m.surface_render_method = "BLENDED"
except AttributeError: halo_m.blend_method = "BLEND"
bpy.ops.mesh.primitive_plane_add(size=6.4, location=(0, 0, -0.1))
halo = bpy.context.object; halo.name = "halo"; halo.data.materials.append(halo_m)

# ---- the transiting umbra ------------------------------------------------------------------
bpy.ops.mesh.primitive_cylinder_add(vertices=256, radius=205 * S, depth=0.02, location=(-7.0, 0.35, 0.1))
moon = bpy.context.object; moon.name = "umbra"; moon.data.materials.append(mat_void)

# ---- stars ---------------------------------------------------------------------------------
rng = random.Random(7)
for i in range(90):
    x, y = rng.uniform(-9, 9), rng.uniform(-5, 5)
    if math.hypot(x, y) < 2.6: continue
    bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=4, radius=rng.choice([0.012, 0.012, 0.02]), location=(x, y, -0.5))
    s = bpy.context.object; s.data.materials.append(mat_star)

# ---- camera: orthographic, straight down the Z axis ----------------------------------------
cam_data = bpy.data.cameras.new("cam"); cam_data.type = "ORTHO"; cam_data.ortho_scale = 10.5
cam = bpy.data.objects.new("cam", cam_data); scene.collection.objects.link(cam)
cam.location = (0, 0, 10); scene.camera = cam

# ---- animation -----------------------------------------------------------------------------
def ease(t):  # smoothstep
    return t * t * (3 - 2 * t)

# umbra transit: frames 1..44, x from -7 to +7 with a slight downward drift, easing
for f in range(1, 45):
    t = ease((f - 1) / 43)
    moon.location = (-7.0 + 14.0 * t, 0.35 - 0.7 * t, 0.1)
    moon.keyframe_insert("location", frame=f)
# corona dims while covered (strength dips), by driving emission strength keyframes
em = next(n for n in mat_gold.node_tree.nodes if n.type == "EMISSION")
for f, s in ((1, 1.0), (16, 1.0), (24, 0.45), (32, 1.04), (72, 1.0)):
    em.inputs["Strength"].default_value = s
    em.inputs["Strength"].keyframe_insert("default_value", frame=f)
# the Z cuts in: frames 46..66, scale from 0 to 1 around the disc centre
for f, s in ((1, 0.0), (45, 0.0), (66, 1.0), (72, 1.0)):
    z_group.scale = (s, s, 1)
    z_group.keyframe_insert("scale", frame=f)
for fc in z_group.animation_data.action.fcurves:
    for kp in fc.keyframe_points:
        kp.interpolation = "BEZIER"

bpy.ops.render.render(animation=True)
print("opener: rendered", FRAMES, "frames to", out_dir)
