import socket
import struct
import netifaces
import time
import psycopg2
import random

# --- CONFIG ---
INTERFACE = 'eth0'
DB_CONFIG = {
    "dbname": "echonet",
    "user": "admin",
    "password": "db@echonet",
    "host": "127.0.0.1"
}
MULTICAST_IP = "224.0.23.0"
ECHONET_PORT = 3610

def parse_props(data):
    props = {}
    if len(data) < 12: return props
    try:
        opc = data[11]
        pos = 12
        for _ in range(opc):
            if pos + 2 > len(data): break
            epc = data[pos:pos+1].hex().upper()
            pdc = data[pos+1]
            props[epc] = data[pos+2 : pos+2+pdc].hex().upper()
            pos += 2 + pdc
    except: pass
    return props

def main():
    try:
        addr_info = netifaces.ifaddresses(INTERFACE).get(netifaces.AF_INET)
        if not addr_info:
            print(f"[-] No IP found on {INTERFACE}"); return
        my_ip = addr_info[0]['addr']
        print(f"[*] Starting Master Sync on {my_ip} (Brightness Disabled)")
    except Exception as e:
        print(f"[-] Interface error: {e}"); return

    # Persistent Socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((my_ip, ECHONET_PORT))
    
    mreq = struct.pack('4s4s', socket.inet_aton(MULTICAST_IP), socket.inet_aton(my_ip))
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_IF, socket.inet_aton(my_ip))

    while True:
        print(f"\n--- Cycle Start: {time.strftime('%H:%M:%S')} ---")
        
        # 1. Discovery
        tid_disc = random.getrandbits(16)
        disc_pkt = b'\x10\x81' + struct.pack('>H', tid_disc) + b'\x0E\xF0\x01\x0E\xF0\x01\x62\x01\xD6\x00'
        sock.sendto(disc_pkt, (MULTICAST_IP, ECHONET_PORT))
        
        nodes = {}
        sock.settimeout(2.0)
        start = time.time()
        while (time.time() - start) < 2.0:
            try:
                data, addr = sock.recvfrom(1024)
                if addr[0] == my_ip: continue
                p = parse_props(data)
                if 'D6' in p:
                    nodes[addr[0]] = [p['D6'][i:i+6] for i in range(2, len(p['D6']), 6)]
            except socket.timeout: break

        # 2. Polling
        db_payload = []
        for ip, instances in nodes.items():
            for eoj in instances:
                if eoj.startswith("0EF0"): continue
                
                tid = random.getrandbits(16)
                # Request Power(80), SetTemp(B3), RoomTemp(BB)
                query = b'\x10\x81' + struct.pack('>H', tid) + b'\x0E\xF0\x01' + bytes.fromhex(eoj) + b'\x62\x03\x80\x00\xB3\x00\xBB\x00'
                sock.sendto(query, (ip, ECHONET_PORT))
                
                try:
                    inner_start = time.time()
                    while (time.time() - inner_start) < 1.2:
                        resp, addr = sock.recvfrom(1024)
                        if addr[0] == ip and resp[2:4] == struct.pack('>H', tid):
                            p = parse_props(resp)
                            if '80' not in p: break
                            
                            pwr = "ON" if p.get('80') == '30' else "OFF"
                            st, rt, dtype = None, None, "Other"
                            
                            if eoj.startswith("0130"):
                                st = int(p['B3'], 16) if 'B3' in p else None
                                rt = int(p['BB'], 16) if 'BB' in p else None
                                dtype = "Air Conditioner"
                            elif eoj.startswith("0290") or eoj.startswith("02A3"):
                                dtype = "Lighting"

                            print(f"    [+] {ip} ({dtype}): Power={pwr}")
                            # Payload has 6 items exactly
                            db_payload.append((ip, eoj, dtype, pwr, st, rt))
                            break
                except: pass
                time.sleep(0.1)

        # 3. Database Sync
        if db_payload:
            try:
                conn = psycopg2.connect(**DB_CONFIG)
                cur = conn.cursor()
                for r in db_payload:
                    # SQL has 6 columns and 6 placeholders
                    cur.execute("""
                        INSERT INTO devices (
                            ip_address, eoj, device_type, power_status, 
                            set_temp, room_temp
                        )
                        VALUES (%s, %s, %s, %s, %s, %s) 
                        ON CONFLICT (ip_address, eoj) DO UPDATE SET
                            power_status = EXCLUDED.power_status,
                            set_temp = EXCLUDED.set_temp, 
                            room_temp = EXCLUDED.room_temp, 
                            device_type = EXCLUDED.device_type,
                            last_updated = CURRENT_TIMESTAMP;
                    """, r)
                conn.commit()
                cur.close()
                conn.close()
                print(f"[✓] DB Updated: {len(db_payload)} devices successfully synced.")
            except Exception as e:
                print(f"[!] DB Error: {e}")
        else:
            print("[-] No device data found to sync.")

        time.sleep(10)

if __name__ == "__main__":
    main()
