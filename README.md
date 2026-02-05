# Distributed File Transfer Application

P2 is a distributed file transfer application originally submitted as coursework for my degree, and is written in Rust. It uses UDP multicast for exchanging control-plane messages and can initiate TCP unicast file transfer. It also contains a parser for the control-plane protocol defined in `protocol.txt`.

# Disclaimer

This should not be used in any production setting! It was designed to be used within local networks only. Security was not the focus of this project.
