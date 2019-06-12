extends Spatial

var laminar

func _ready():
	laminar = get_node("/root/Global").laminar

func on_network_received(data):
    print("Game node: ", data)