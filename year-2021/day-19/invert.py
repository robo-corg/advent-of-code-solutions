import numpy as np
from numpy.linalg import inv

# B*x = y
# C*B = A

# C = A*B^-1

a = np.array([
    [-665, -529, 392, 1],
    [-678, -709, 356, 1],
    [592, -430, 681, 1],
    [-849, -421, -700, 1]
])

b = np.array([
    [627, 391, 574, 1],
    [640, 355, 394, 1],
    [-630, 680, 673, 1],
    [811, -701, 682, 1]
])

#b_inv = inv(np.matrix(b))

#print(inv(np.matrix(b)))

#transform = np.matrix(a) * b_inv

print(b)
print(a - b)
#print(a - b)


# b[:, [2, 0]] = b[:, [0, 2]]
# print(b)
# print(a - b)


# b[:, [2, 1]] = b[:, [1, 2]]
# print(b)
# print(a - b)


r = np.array([
    [-1, 0, 0, 0],
    [0, 0, 1, 0],
    [0, 1, 0, 0],
    [0, 0, 0, 1]
])

print(np.matrix(b)*np.matrix(r))

print(a - np.matrix(b)*np.matrix(r))

#a - b*np.matrix(r)

# # print(np.matrix(b)*np.matrix(r))

d = [-38, -1103, 1]

t1 = np.array([
    [1, 0, 0, d[0]],
    [0, 1, 0, d[1]],
    [0, 0, 1, d[2]],
    [0, 0, 0, 1]
])

t2 = np.array([
    [1, 0, 0, 0],
    [0, 1, 0, 0],
    [0, 0, 1, 0],
    [d[0], d[1], d[2], 1]
])


print(np.matrix(b)*(np.matrix(r)*np.matrix(t2)))

b0 = np.array([627, 391, 574, 1])

print(np.matrix(b0)*(np.matrix(r)*np.matrix(t2)))

print((np.matrix(r)*np.matrix(t2)))




# a = b * r * t2
# a * b^1 = r * t2


b_inv = inv(np.matrix(b))

print(b_inv * a)

print(np.around(b_inv * a))

print(np.matrix(b) * np.around(b_inv * a))