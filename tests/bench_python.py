import time

class Point:
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

def main():
    print("Iniciando benchmark...")
    start = time.time()
    total = 0.0
    
    for _ in range(5000000):
        # Aloca classe Point na Heap constantemente
        p = Point(1.0, 2.0, 3.0)
        total += p.x + p.y + p.z
        
    end = time.time()
    print(f"Resultado Python: {total}")
    print(f"Tempo Python: {end - start:.4f} s")

if __name__ == "__main__":
    main()
