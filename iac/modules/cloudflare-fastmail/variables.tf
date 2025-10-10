variable "zone_id" {
  type = string
}

variable "domain_name" {
  type = string
}

variable "ttl" {
  type    = number
  default = 3600
}
