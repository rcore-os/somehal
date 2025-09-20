#!/usr/bin/env python3
"""
EFI镜像结构检查工具
模拟EDK2 PeCoffLoaderGetImageInfo的验证逻辑
检查PE32+结构的各个关键字段
"""

import sys
import struct
from typing import Dict, List, Tuple, Optional

class EFIImageChecker:
    def __init__(self, file_path: str):
        self.file_path = file_path
        self.data = None
        self.pe_offset = 0
        self.errors = []
        self.warnings = []
        
    def load_file(self) -> bool:
        """加载EFI文件"""
        try:
            with open(self.file_path, 'rb') as f:
                self.data = f.read()
            return True
        except Exception as e:
            self.errors.append(f"无法读取文件: {e}")
            return False
    
    def check_dos_header(self) -> bool:
        """检查DOS头部"""
        print("=== 检查DOS头部 ===")
        
        if len(self.data) < 64:
            self.errors.append("文件太小，无法包含DOS头部")
            return False
            
        # 检查MZ签名
        if self.data[:2] != b'MZ':
            self.errors.append(f"无效的DOS签名: {self.data[:2].hex()} (应该是 4D5A)")
            return False
        print("✓ DOS签名正确: MZ")
        
        # 获取PE头偏移
        self.pe_offset = struct.unpack('<L', self.data[0x3C:0x40])[0]
        print(f"✓ PE头偏移: 0x{self.pe_offset:X}")
        
        if self.pe_offset >= len(self.data):
            self.errors.append(f"PE头偏移超出文件范围: 0x{self.pe_offset:X} >= 0x{len(self.data):X}")
            return False
            
        return True
    
    def check_pe_header(self) -> Dict:
        """检查PE头部"""
        print("=== 检查PE头部 ===")
        
        if self.pe_offset + 4 > len(self.data):
            self.errors.append("PE头偏移超出文件范围")
            return {}
            
        # 检查PE签名
        pe_sig = self.data[self.pe_offset:self.pe_offset+4]
        if pe_sig != b'PE\x00\x00':
            self.errors.append(f"无效的PE签名: {pe_sig.hex()} (应该是 50450000)")
            return {}
        print("✓ PE签名正确: PE\\0\\0")
        
        # COFF头部
        coff_offset = self.pe_offset + 4
        if coff_offset + 20 > len(self.data):
            self.errors.append("COFF头部超出文件范围")
            return {}
            
        coff_data = struct.unpack('<HHLLLHH', self.data[coff_offset:coff_offset+20])
        
        pe_info = {
            'machine': coff_data[0],
            'sections': coff_data[1], 
            'timestamp': coff_data[2],
            'symbol_table': coff_data[3],
            'symbols': coff_data[4],
            'opt_header_size': coff_data[5],
            'characteristics': coff_data[6]
        }
        
        print(f"✓ 机器类型: 0x{pe_info['machine']:X}", end="")
        if pe_info['machine'] == 0x6264:
            print(" (LoongArch64)")
        else:
            print(f" (未知)")
            self.warnings.append(f"机器类型不是LoongArch64: 0x{pe_info['machine']:X}")
            
        print(f"✓ 段数量: {pe_info['sections']}")
        print(f"✓ 可选头大小: {pe_info['opt_header_size']} bytes")
        print(f"✓ 特征标志: 0x{pe_info['characteristics']:X}")
        
        return pe_info
    
    def check_optional_header(self, pe_info: Dict) -> Dict:
        """检查可选头部(PE32+)"""
        print("=== 检查可选头部(PE32+) ===")
        
        opt_offset = self.pe_offset + 24  # PE签名(4) + COFF头(20)
        opt_size = pe_info['opt_header_size']
        
        if opt_offset + opt_size > len(self.data):
            self.errors.append("可选头部超出文件范围")
            return {}
            
        # 检查魔数
        magic = struct.unpack('<H', self.data[opt_offset:opt_offset+2])[0]
        if magic != 0x020B:
            self.errors.append(f"不是PE32+格式: 0x{magic:X} (应该是 020B)")
            return {}
        print("✓ 格式: PE32+")
        
        # 解析关键字段
        opt_data = self.data[opt_offset:opt_offset+opt_size]
        
        opt_info = {}
        try:
            # PE32+字段偏移
            opt_info['magic'] = struct.unpack('<H', opt_data[0:2])[0]
            opt_info['linker_major'] = struct.unpack('<B', opt_data[2:3])[0] 
            opt_info['linker_minor'] = struct.unpack('<B', opt_data[3:4])[0]
            opt_info['code_size'] = struct.unpack('<L', opt_data[4:8])[0]
            opt_info['data_size'] = struct.unpack('<L', opt_data[8:12])[0]
            opt_info['bss_size'] = struct.unpack('<L', opt_data[12:16])[0]
            opt_info['entry_point'] = struct.unpack('<L', opt_data[16:20])[0]
            opt_info['code_base'] = struct.unpack('<L', opt_data[20:24])[0]
            opt_info['image_base'] = struct.unpack('<Q', opt_data[24:32])[0]
            opt_info['section_align'] = struct.unpack('<L', opt_data[32:36])[0]
            opt_info['file_align'] = struct.unpack('<L', opt_data[36:40])[0]
            opt_info['os_major'] = struct.unpack('<H', opt_data[40:42])[0]
            opt_info['os_minor'] = struct.unpack('<H', opt_data[42:44])[0]
            opt_info['img_major'] = struct.unpack('<H', opt_data[44:46])[0]
            opt_info['img_minor'] = struct.unpack('<H', opt_data[46:48])[0]
            opt_info['subsys_major'] = struct.unpack('<H', opt_data[48:50])[0]
            opt_info['subsys_minor'] = struct.unpack('<H', opt_data[50:52])[0]
            opt_info['win32_version'] = struct.unpack('<L', opt_data[52:56])[0]
            opt_info['image_size'] = struct.unpack('<L', opt_data[56:60])[0]
            opt_info['header_size'] = struct.unpack('<L', opt_data[60:64])[0]
            opt_info['checksum'] = struct.unpack('<L', opt_data[64:68])[0]
            opt_info['subsystem'] = struct.unpack('<H', opt_data[68:70])[0]
            opt_info['dll_characteristics'] = struct.unpack('<H', opt_data[70:72])[0]
            opt_info['stack_reserve'] = struct.unpack('<Q', opt_data[72:80])[0]
            opt_info['stack_commit'] = struct.unpack('<Q', opt_data[80:88])[0]
            opt_info['heap_reserve'] = struct.unpack('<Q', opt_data[88:96])[0]
            opt_info['heap_commit'] = struct.unpack('<Q', opt_data[96:104])[0]
            opt_info['loader_flags'] = struct.unpack('<L', opt_data[104:108])[0]
            opt_info['rva_sizes'] = struct.unpack('<L', opt_data[108:112])[0]
            
        except struct.error as e:
            self.errors.append(f"解析可选头部失败: {e}")
            return {}
        
        # 验证关键字段
        print(f"✓ 代码大小: 0x{opt_info['code_size']:X}")
        print(f"✓ 数据大小: 0x{opt_info['data_size']:X}")
        print(f"✓ 入口点: 0x{opt_info['entry_point']:X}")
        print(f"✓ 镜像基址: 0x{opt_info['image_base']:X}")
        print(f"✓ 段对齐: 0x{opt_info['section_align']:X}")
        print(f"✓ 文件对齐: 0x{opt_info['file_align']:X}")
        print(f"✓ 镜像大小: 0x{opt_info['image_size']:X}")
        print(f"✓ 头部大小: 0x{opt_info['header_size']:X}")
        print(f"✓ 校验和: 0x{opt_info['checksum']:X}")
        print(f"✓ 子系统: {opt_info['subsystem']}", end="")
        if opt_info['subsystem'] == 10:
            print(" (EFI应用程序)")
        else:
            print(" (未知)")
            self.warnings.append(f"子系统不是EFI应用程序: {opt_info['subsystem']}")
        
        print(f"✓ RVA数量: {opt_info['rva_sizes']}")
        
        # 验证关键约束
        self.validate_optional_header(opt_info, pe_info)
        
        return opt_info
    
    def validate_optional_header(self, opt_info: Dict, pe_info: Dict):
        """验证可选头部的约束条件"""
        print("=== 验证可选头部约束 ===")
        
        # 1. 检查头部大小
        section_table_offset = self.pe_offset + 24 + pe_info['opt_header_size']
        if opt_info['header_size'] < section_table_offset:
            self.errors.append(f"头部大小太小: 0x{opt_info['header_size']:X} < 0x{section_table_offset:X}")
        elif opt_info['header_size'] >= opt_info['image_size']:
            self.errors.append(f"头部大小超过镜像大小: 0x{opt_info['header_size']:X} >= 0x{opt_info['image_size']:X}")
        else:
            print(f"✓ 头部大小验证通过")
            
        # 2. 检查段表空间
        section_table_size = pe_info['sections'] * 40  # 每个段表项40字节
        available_space = opt_info['header_size'] - section_table_offset
        if section_table_size > available_space:
            self.errors.append(f"段表空间不足: 需要{section_table_size}字节，可用{available_space}字节")
        else:
            print(f"✓ 段表空间验证通过")
            
        # 3. 检查入口点
        if opt_info['entry_point'] >= opt_info['image_size']:
            self.errors.append(f"入口点超出镜像范围: 0x{opt_info['entry_point']:X} >= 0x{opt_info['image_size']:X}")
        else:
            print(f"✓ 入口点验证通过")
            
        # 4. 检查对齐
        if opt_info['section_align'] == 0 or (opt_info['section_align'] & (opt_info['section_align'] - 1)) != 0:
            self.warnings.append(f"段对齐不是2的幂: 0x{opt_info['section_align']:X}")
        
        if opt_info['file_align'] == 0 or (opt_info['file_align'] & (opt_info['file_align'] - 1)) != 0:
            self.warnings.append(f"文件对齐不是2的幂: 0x{opt_info['file_align']:X}")
    
    def check_section_table(self, pe_info: Dict, opt_info: Dict) -> List[Dict]:
        """检查段表"""
        print("=== 检查段表 ===")
        
        section_offset = self.pe_offset + 24 + pe_info['opt_header_size']
        sections = []
        
        for i in range(pe_info['sections']):
            sec_offset = section_offset + i * 40
            if sec_offset + 40 > len(self.data):
                self.errors.append(f"段表项{i}超出文件范围")
                continue
                
            sec_data = self.data[sec_offset:sec_offset+40]
            
            section = {
                'name': sec_data[:8].rstrip(b'\x00').decode('ascii', errors='ignore'),
                'virtual_size': struct.unpack('<L', sec_data[8:12])[0],
                'virtual_addr': struct.unpack('<L', sec_data[12:16])[0],
                'raw_size': struct.unpack('<L', sec_data[16:20])[0],
                'raw_ptr': struct.unpack('<L', sec_data[20:24])[0],
                'reloc_ptr': struct.unpack('<L', sec_data[24:28])[0],
                'line_ptr': struct.unpack('<L', sec_data[28:32])[0],
                'reloc_count': struct.unpack('<H', sec_data[32:34])[0],
                'line_count': struct.unpack('<H', sec_data[34:36])[0],
                'characteristics': struct.unpack('<L', sec_data[36:40])[0]
            }
            
            sections.append(section)
            
            print(f"段{i}: {section['name']}")
            print(f"  虚拟大小: 0x{section['virtual_size']:X}")
            print(f"  虚拟地址: 0x{section['virtual_addr']:X}")
            print(f"  原始大小: 0x{section['raw_size']:X}")
            print(f"  原始指针: 0x{section['raw_ptr']:X}")
            print(f"  特征: 0x{section['characteristics']:X}")
            
            # 验证段数据
            if section['raw_ptr'] + section['raw_size'] > len(self.data):
                self.errors.append(f"段{i}({section['name']})数据超出文件范围")
            
            if section['virtual_addr'] + section['virtual_size'] > opt_info['image_size']:
                self.errors.append(f"段{i}({section['name']})超出镜像范围")
        
        return sections
    
    def check_file_consistency(self):
        """检查文件整体一致性"""
        print("=== 检查文件一致性 ===")
        
        # 检查文件是否被截断
        expected_size = len(self.data)
        print(f"✓ 文件大小: {expected_size} 字节")
        
        # 检查是否有多余数据
        if expected_size % 512 != 0:
            self.warnings.append(f"文件大小不是512字节对齐: {expected_size}")
    
    def run_check(self) -> bool:
        """运行完整检查"""
        print(f"检查EFI镜像: {self.file_path}")
        print("=" * 50)
        
        if not self.load_file():
            return False
            
        if not self.check_dos_header():
            return False
            
        pe_info = self.check_pe_header()
        if not pe_info:
            return False
            
        opt_info = self.check_optional_header(pe_info)
        if not opt_info:
            return False
            
        sections = self.check_section_table(pe_info, opt_info)
        
        self.check_file_consistency()
        
        # 报告结果
        print("\n" + "=" * 50)
        print("检查结果:")
        
        if self.errors:
            print(f"❌ 发现 {len(self.errors)} 个错误:")
            for i, error in enumerate(self.errors, 1):
                print(f"  {i}. {error}")
        
        if self.warnings:
            print(f"⚠️  发现 {len(self.warnings)} 个警告:")
            for i, warning in enumerate(self.warnings, 1):
                print(f"  {i}. {warning}")
        
        if not self.errors and not self.warnings:
            print("✅ 所有检查通过！")
        elif not self.errors:
            print("✅ 结构验证通过（有警告）")
        
        success = len(self.errors) == 0
        return success

def main():
    if len(sys.argv) != 2:
        print("用法: python3 check_efi.py <efi_file>")
        sys.exit(1)
        
    efi_file = sys.argv[1]
    checker = EFIImageChecker(efi_file)
    
    success = checker.run_check()
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()