extends Node

var laminar

var username
var network_id

func _ready():
    laminar = get_node("/root/Laminar")

func init_client(username, address):
    laminar.init_client(address, self as Node)
