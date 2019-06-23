extends Spatial

var laminar
var global
onready var new_player = preload("res://Player.tscn")

func _ready():
	laminar = get_node("/root/Global").laminar
	laminar.set_root("/root/Game")
	global = get_node("/root/Global")

func _on_net_timed_out():
	print("Game: Connection to server timed out.")
	get_tree().change_scene("res://MainMenu.tscn")

func _on_net_pos_relayed(data):
	pass
	#print("Pos relayed:", data[1])
	
func _on_net_set_others_pos(data):
	var id = data[0]
	var pos = data[1]
	
	if id != global.client_id:
		var possible_player_path = "player_" + String(id)
		if self.has_node(NodePath(possible_player_path)):
			var other_player = get_node(NodePath(possible_player_path))
			other_player.global_transform.origin = pos
			print("player_", id, " moved to ", pos)
		else:
			var player = new_player.instance()
			add_child(player)
			player.set_name("player_" + String(id))
			print("Instanced new player!!!!!")
			player.global_transform.origin = pos