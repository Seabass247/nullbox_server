extends Spatial

var laminar
var global
onready var new_player = preload("res://Player.tscn")

func _ready():
    laminar = get_node("/root/Global").laminar
    global = get_node("/root/Global")

func on_network_received(data):
    var pack = data.split(":")
    if (pack[0] == "upd_ply"):
        var fields = pack[1].split(";")
        for field in fields:
            if field == "":
                continue
            var subfield = field.split("=")
            if int(subfield[0]) != global.network_id:
                var possible_player_path = "player_" + subfield[0]
                var pos_comps = subfield[1].split(",")
                var pos = Vector3(float(pos_comps[0]), float(pos_comps[1]), float(pos_comps[2]))
                
                if self.has_node(NodePath(possible_player_path)):
                    var other_player = get_node(NodePath(possible_player_path))
                    other_player.global_transform.origin = pos
                else:
                    var player = new_player.instance()
                    add_child(player)
                    player.set_name("player_" + subfield[0])
                    player.global_transform.origin = pos





