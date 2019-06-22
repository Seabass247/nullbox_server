extends Spatial

var laminar
var global
onready var new_player = preload("res://Player.tscn")

func _ready():
	laminar = get_node("/root/Global").laminar
	laminar.set_root("/root/Game")
	global = get_node("/root/Global")

func on_network_received(data):
    if (data[0][0] == "upd_ply"):
        for field in data:
            if field[0] == "upd_ply":
                continue
            var id = field[0]  
            var pos_x = float(field[1])
            var pos_y = float(field[2])
            var pos_z = float(field[3])
            if int(id) != global.network_id:
                var possible_player_path = "player_" + id
                var pos = Vector3(pos_x, pos_y, pos_z)
                if self.has_node(NodePath(possible_player_path)):
                    var other_player = get_node(NodePath(possible_player_path))
                    other_player.global_transform.origin = pos
                else:
                    var player = new_player.instance()
                    add_child(player)
                    player.set_name("player_" + id)
                    player.global_transform.origin = pos

func _on_net_set_others_pos(data):
	var id = data[0]
	var pos = data[1]
	