import struct
import zlib

def remove_white_bg(input_path, output_path, threshold=235):
    with open(input_path, 'rb') as f:
        raw = f.read()

    pos = 8
    chunks = []
    while pos < len(raw):
        length = struct.unpack('>I', raw[pos:pos+4])[0]
        chunk_type = raw[pos+4:pos+8]
        chunk_data = raw[pos+8:pos+8+length]
        chunk_crc = raw[pos+8+length:pos+12+length]
        chunks.append((chunk_type, chunk_data, chunk_crc))
        pos += 12 + length

    ihdr_type, ihdr_data, ihdr_crc = chunks[0]
    width = struct.unpack('>I', ihdr_data[0:4])[0]
    height = struct.unpack('>I', ihdr_data[4:8])[0]
    bit_depth = ihdr_data[8]
    color_type = ihdr_data[9]
    print(f"Image: {width}x{height}, bit_depth={bit_depth}, color_type={color_type}")

    idat_data = b''
    for ct, cd, cc in chunks:
        if ct == b'IDAT':
            idat_data += cd

    decompressed = zlib.decompress(idat_data)

    if color_type == 2:
        bpp = 3
    elif color_type == 6:
        bpp = 4
    else:
        bpp = 1

    new_rows = []
    pos = 0
    for y in range(height):
        filter_byte = decompressed[pos]
        pos += 1
        row = bytearray(decompressed[pos:pos + width * bpp])
        pos += width * bpp

        # Unfilter
        if filter_byte == 1:  # Sub
            for x in range(bpp, len(row)):
                row[x] = (row[x] + row[x - bpp]) & 0xFF
        elif filter_byte == 2:  # Up
            if y > 0:
                prev_row = new_rows[y-1][1:]
                for x in range(len(row)):
                    row[x] = (row[x] + prev_row[x]) & 0xFF
        elif filter_byte == 3:  # Average
            prev_row = new_rows[y-1][1:] if y > 0 else None
            for x in range(len(row)):
                a = row[x - bpp] if x >= bpp else 0
                b_val = prev_row[x] if prev_row else 0
                row[x] = (row[x] + (a + b_val) // 2) & 0xFF
        elif filter_byte == 4:  # Paeth
            prev_row = new_rows[y-1][1:] if y > 0 else None
            for x in range(len(row)):
                a = row[x - bpp] if x >= bpp else 0
                b_val = prev_row[x] if prev_row else 0
                c = prev_row[x - bpp] if (x >= bpp and prev_row) else 0
                p = a + b_val - c
                pa = abs(p - a)
                pb = abs(p - b_val)
                pc = abs(p - c)
                if pa <= pb and pa <= pc:
                    pr = a
                elif pb <= pc:
                    pr = b_val
                else:
                    pr = c
                row[x] = (row[x] + pr) & 0xFF

        # Make near-white pixels transparent, convert to RGBA
        new_row = bytearray()
        for x in range(width):
            if color_type == 2:
                r = row[x*3]
                g = row[x*3+1]
                b = row[x*3+2]
                orig_a = 255
            else:
                r = row[x*4]
                g = row[x*4+1]
                b = row[x*4+2]
                orig_a = row[x*4+3]

            if r > threshold and g > threshold and b > threshold:
                new_row.extend([r, g, b, 0])
            else:
                new_row.extend([r, g, b, orig_a])
        new_rows.append(bytearray([0]) + new_row)

    new_decompressed = b''.join(new_rows)
    new_compressed = zlib.compress(new_decompressed)

    new_png = b'\x89PNG\r\n\x1a\n'

    new_ihdr = struct.pack('>IIBBBBB', width, height, bit_depth, 6, 0, 0, 0)
    crc = zlib.crc32(b'IHDR' + new_ihdr) & 0xFFFFFFFF
    new_png += struct.pack('>I', 13) + b'IHDR' + new_ihdr + struct.pack('>I', crc)

    crc = zlib.crc32(b'IDAT' + new_compressed) & 0xFFFFFFFF
    new_png += struct.pack('>I', len(new_compressed)) + b'IDAT' + new_compressed + struct.pack('>I', crc)

    crc = zlib.crc32(b'IEND') & 0xFFFFFFFF
    new_png += struct.pack('>I', 0) + b'IEND' + struct.pack('>I', crc)

    with open(output_path, 'wb') as f:
        f.write(new_png)
    print(f"Saved to {output_path}")

remove_white_bg('desktop-pet.png', 'desktop-pet.png', threshold=235)
