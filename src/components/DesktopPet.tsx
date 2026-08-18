import { useEffect, useRef, useState } from "react";

const PET_WIDTH = 200;
const PET_HEIGHT = 260;

export default function DesktopPet() {
  const petRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [isBouncing, setIsBouncing] = useState(false);
  const dragOffset = useRef({ x: 0, y: 0 });

  useEffect(() => {
    // 将桌宠放在屏幕右下角
    const x = window.innerWidth - PET_WIDTH - 20;
    const y = window.innerHeight - PET_HEIGHT - 20;
    setPosition({ x, y });
  }, []);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const newX = e.clientX - dragOffset.current.x;
      const newY = e.clientY - dragOffset.current.y;
      setPosition({ x: newX, y: newY });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      // 拖拽结束后弹跳一下
      setIsBouncing(true);
      setTimeout(() => setIsBouncing(false), 500);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging]);

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    dragOffset.current = {
      x: e.clientX - position.x,
      y: e.clientY - position.y,
    };
  };

  return (
    <div
      ref={petRef}
      className="desktop-pet"
      style={{
        left: position.x,
        top: position.y,
        cursor: isDragging ? "grabbing" : "grab",
      }}
      onMouseDown={handleMouseDown}
    >
      <div className={`desktop-pet-inner ${isBouncing ? "pet-bounce" : ""}`}>
        <img
          src="/desktop-pet.png"
          alt="Desktop Pet"
          className="desktop-pet-image"
          draggable={false}
        />
        <div className="desktop-pet-glow" />
      </div>
    </div>
  );
}
