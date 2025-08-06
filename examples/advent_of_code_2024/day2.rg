import 'utils/day.rg'

(file, p1, p2) = day_io 2

file.lines.map { |line| line.split(' ').map{ |l| l.to_i } }