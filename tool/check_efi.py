#!/usr/bin/env python3
"""
EFI镜像结构检查工具
模拟EDK2 PeCoffLoaderGetImageInfo的验证逻辑
检查PE32+结构的各个关键字段
支持与参考EFI文件对比验证
"""

import sys
import struct
import os
from typing import Dict, List, Tuple, Optional

class EFIImageChecker:
    def __init__(self, file_path: str, reference_path: Optional[str] = None):
        self.file_path = file_path
        self.reference_path = reference_path
        self.data = None
        self.ref_data = None
        self.pe_offset = 0
        self.ref_pe_offset = 0
        self.errors = []
        self.warnings = []
        self.info = []
        
    def load_file(self) -> bool:
        """加载EFI文件"""
        try:
            with open(self.file_path, 'rb') as f:
                self.data = f.read()
            print(f"✓ 成功加载目标文件: {self.file_path} ({len(self.data)} 字节)")
            
            # 如果有参考文件，也加载它
            if self.reference_path:
                try:
                    with open(self.reference_path, 'rb') as f:
                        self.ref_data = f.read()
                    print(f"✓ 成功加载参考文件: {self.reference_path} ({len(self.ref_data)} 字节)")
                except Exception as e:
                    self.warnings.append(f"无法读取参考文件: {e}")
                    self.reference_path = None
            
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
        
        # 如果有参考文件，对比DOS头
        if self.ref_data:
            self.compare_dos_header()
            
        return True
    
    def compare_dos_header(self):
        """对比DOS头部"""
        if len(self.ref_data) < 64:
            return
            
        self.ref_pe_offset = struct.unpack('<L', self.ref_data[0x3C:0x40])[0]
        
        if self.pe_offset != self.ref_pe_offset:
            self.info.append(f"PE头偏移不同: 目标=0x{self.pe_offset:X}, 参考=0x{self.ref_pe_offset:X}")
        else:
            self.info.append(f"PE头偏移一致: 0x{self.pe_offset:X}")
    
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
        elif pe_info['machine'] == 0x8664:
            print(" (x86_64)")
        elif pe_info['machine'] == 0xAA64:
            print(" (AArch64)")
        else:
            print(f" (未知)")
            self.warnings.append(f"未知机器类型: 0x{pe_info['machine']:X}")
            
        print(f"✓ 段数量: {pe_info['sections']}")
        print(f"✓ 可选头大小: {pe_info['opt_header_size']} bytes")
        print(f"✓ 特征标志: 0x{pe_info['characteristics']:X}")
        
        # 如果有参考文件，对比PE头
        if self.ref_data:
            self.compare_pe_header(pe_info)
        
        return pe_info
    
    def compare_pe_header(self, pe_info: Dict):
        """对比PE头部"""
        if self.ref_pe_offset + 24 > len(self.ref_data):
            return
            
        ref_coff_offset = self.ref_pe_offset + 4
        ref_coff_data = struct.unpack('<HHLLLHH', self.ref_data[ref_coff_offset:ref_coff_offset+20])
        
        ref_pe_info = {
            'machine': ref_coff_data[0],
            'sections': ref_coff_data[1],
            'opt_header_size': ref_coff_data[5],
            'characteristics': ref_coff_data[6]
        }
        
        if pe_info['machine'] != ref_pe_info['machine']:
            self.warnings.append(f"机器类型不同: 目标=0x{pe_info['machine']:X}, 参考=0x{ref_pe_info['machine']:X}")
        
        if pe_info['sections'] != ref_pe_info['sections']:
            self.info.append(f"段数量不同: 目标={pe_info['sections']}, 参考={ref_pe_info['sections']}")
        
        if pe_info['opt_header_size'] != ref_pe_info['opt_header_size']:
            self.info.append(f"可选头大小不同: 目标={pe_info['opt_header_size']}, 参考={ref_pe_info['opt_header_size']}")
        
        if pe_info['characteristics'] != ref_pe_info['characteristics']:
            self.info.append(f"特征标志不同: 目标=0x{pe_info['characteristics']:X}, 参考=0x{ref_pe_info['characteristics']:X}")
    
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
        elif opt_info['subsystem'] == 11:
            print(" (EFI引导服务驱动)")
        elif opt_info['subsystem'] == 12:
            print(" (EFI运行时驱动)")
        else:
            print(" (未知)")
            self.warnings.append(f"子系统不是EFI类型: {opt_info['subsystem']}")
        
        print(f"✓ RVA数量: {opt_info['rva_sizes']}")
        
        # 检查数据目录表
        self.check_data_directories(opt_data, opt_info)
        
        # 验证关键约束
        self.validate_optional_header(opt_info, pe_info)
        
        # 如果有参考文件，对比可选头部
        if self.ref_data:
            self.compare_optional_header(opt_info, pe_info)
        
        return opt_info
    
    def check_data_directories(self, opt_data: bytes, opt_info: Dict):
        """检查数据目录表"""
        print("=== 检查数据目录表 ===")
        
        # 数据目录表从偏移112开始，每个条目8字节(RVA + Size)
        dir_offset = 112
        rva_count = opt_info['rva_sizes']
        
        if rva_count > 16:
            self.warnings.append(f"RVA数量过多: {rva_count} (通常不超过16)")
            rva_count = 16
        
        dir_names = [
            "导出表", "导入表", "资源表", "异常表", "证书表", "基址重定位表",
            "调试", "架构", "全局指针", "TLS表", "加载配置表", "绑定导入",
            "IAT", "延迟导入描述符", "COM+运行时头", "保留"
        ]
        
        for i in range(rva_count):
            if dir_offset + 8 > len(opt_data):
                break
                
            rva = struct.unpack('<L', opt_data[dir_offset:dir_offset+4])[0]
            size = struct.unpack('<L', opt_data[dir_offset+4:dir_offset+8])[0]
            
            name = dir_names[i] if i < len(dir_names) else f"未知{i}"
            
            if rva != 0 or size != 0:
                print(f"  {name}: RVA=0x{rva:X}, Size=0x{size:X}")
                
                # 验证RVA范围
                if rva != 0 and rva >= opt_info['image_size']:
                    self.errors.append(f"{name}的RVA超出镜像范围: 0x{rva:X}")
                
                if rva != 0 and size != 0 and rva + size > opt_info['image_size']:
                    self.errors.append(f"{name}的数据超出镜像范围: 0x{rva:X}+0x{size:X}")
            
            dir_offset += 8
    
    def compare_optional_header(self, opt_info: Dict, pe_info: Dict):
        """对比可选头部"""
        if not self.ref_data:
            return
            
        ref_opt_offset = self.ref_pe_offset + 24
        ref_opt_size = pe_info['opt_header_size']  # 假设大小相同，实际应该从参考文件读取
        
        if ref_opt_offset + ref_opt_size > len(self.ref_data):
            return
            
        ref_opt_data = self.ref_data[ref_opt_offset:ref_opt_offset+ref_opt_size]
        
        try:
            ref_opt_info = {
                'code_size': struct.unpack('<L', ref_opt_data[4:8])[0],
                'data_size': struct.unpack('<L', ref_opt_data[8:12])[0],
                'entry_point': struct.unpack('<L', ref_opt_data[16:20])[0],
                'image_base': struct.unpack('<Q', ref_opt_data[24:32])[0],
                'section_align': struct.unpack('<L', ref_opt_data[32:36])[0],
                'file_align': struct.unpack('<L', ref_opt_data[36:40])[0],
                'image_size': struct.unpack('<L', ref_opt_data[56:60])[0],
                'header_size': struct.unpack('<L', ref_opt_data[60:64])[0],
                'subsystem': struct.unpack('<H', ref_opt_data[68:70])[0],
            }
            
            print("=== 与参考文件对比 ===")
            
            key_fields = ['code_size', 'data_size', 'image_size', 'section_align', 'file_align']
            for field in key_fields:
                if opt_info[field] != ref_opt_info[field]:
                    ratio = opt_info[field] / ref_opt_info[field] if ref_opt_info[field] != 0 else 0
                    self.info.append(f"{field}不同: 目标=0x{opt_info[field]:X}, 参考=0x{ref_opt_info[field]:X} (比例={ratio:.2f})")
                    
        except Exception as e:
            self.warnings.append(f"对比可选头部失败: {e}")
    
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
            
            # 详细特征分析
            char_flags = []
            if section['characteristics'] & 0x20:
                char_flags.append("CODE")
            if section['characteristics'] & 0x40:
                char_flags.append("INITIALIZED_DATA")
            if section['characteristics'] & 0x80:
                char_flags.append("UNINITIALIZED_DATA")
            if section['characteristics'] & 0x20000000:
                char_flags.append("EXECUTABLE")
            if section['characteristics'] & 0x40000000:
                char_flags.append("READABLE")
            if section['characteristics'] & 0x80000000:
                char_flags.append("WRITABLE")
            
            if char_flags:
                print(f"    权限: {' | '.join(char_flags)}")
            
            # 验证段数据一致性
            self.validate_section(section, opt_info, i)
        
        # 验证段之间的关系
        self.validate_sections_relationship(sections, opt_info)
        
        # 如果有参考文件，对比段表
        if self.ref_data:
            self.compare_sections(sections, pe_info)
        
        return sections
    
    def validate_section(self, section: Dict, opt_info: Dict, index: int):
        """验证单个段的一致性"""
        name = section['name']
        
        # 1. 检查文件范围
        if section['raw_ptr'] != 0 and section['raw_size'] != 0:
            file_end = section['raw_ptr'] + section['raw_size']
            if file_end > len(self.data):
                self.errors.append(f"段{index}({name})数据超出文件范围: 0x{file_end:X} > 0x{len(self.data):X}")
        
        # 2. 检查虚拟地址范围
        if section['virtual_addr'] + section['virtual_size'] > opt_info['image_size']:
            self.errors.append(f"段{index}({name})超出镜像范围: 0x{section['virtual_addr']:X}+0x{section['virtual_size']:X} > 0x{opt_info['image_size']:X}")
        
        # 3. 检查对齐
        if section['virtual_addr'] % opt_info['section_align'] != 0:
            self.warnings.append(f"段{index}({name})虚拟地址未对齐: 0x{section['virtual_addr']:X} % 0x{opt_info['section_align']:X} != 0")
        
        if section['raw_ptr'] != 0 and section['raw_ptr'] % opt_info['file_align'] != 0:
            self.warnings.append(f"段{index}({name})文件指针未对齐: 0x{section['raw_ptr']:X} % 0x{opt_info['file_align']:X} != 0")
        
        # 4. 检查大小一致性
        if section['raw_size'] > 0 and section['virtual_size'] > 0:
            if section['raw_size'] > section['virtual_size'] * 2:
                self.warnings.append(f"段{index}({name})原始大小远大于虚拟大小: 0x{section['raw_size']:X} vs 0x{section['virtual_size']:X}")
        
        # 5. 检查BSS段
        if section['raw_size'] == 0 and section['virtual_size'] > 0:
            if not (section['characteristics'] & 0x80):  # UNINITIALIZED_DATA
                self.info.append(f"段{index}({name})可能是BSS段但未标记为未初始化数据")
    
    def validate_sections_relationship(self, sections: List[Dict], opt_info: Dict):
        """验证段之间的关系"""
        print("=== 验证段关系 ===")
        
        # 检查段是否重叠
        for i, sec1 in enumerate(sections):
            for j, sec2 in enumerate(sections[i+1:], i+1):
                # 检查虚拟地址重叠
                if (sec1['virtual_addr'] < sec2['virtual_addr'] + sec2['virtual_size'] and
                    sec2['virtual_addr'] < sec1['virtual_addr'] + sec1['virtual_size']):
                    self.errors.append(f"段{i}({sec1['name']})和段{j}({sec2['name']})虚拟地址重叠")
                
                # 检查文件地址重叠（如果都有文件数据）
                if (sec1['raw_size'] > 0 and sec2['raw_size'] > 0 and
                    sec1['raw_ptr'] < sec2['raw_ptr'] + sec2['raw_size'] and
                    sec2['raw_ptr'] < sec1['raw_ptr'] + sec1['raw_size']):
                    self.errors.append(f"段{i}({sec1['name']})和段{j}({sec2['name']})文件数据重叠")
        
        # 计算实际代码和数据大小
        total_code_size = 0
        total_data_size = 0
        
        for section in sections:
            if section['characteristics'] & 0x20:  # CODE
                total_code_size += section['virtual_size']
            elif section['characteristics'] & 0x40:  # INITIALIZED_DATA
                total_data_size += section['virtual_size']
        
        # 与头部声明的大小对比
        if total_code_size != opt_info['code_size']:
            self.warnings.append(f"实际代码大小与头部不匹配: 0x{total_code_size:X} vs 0x{opt_info['code_size']:X}")
        
        print(f"✓ 实际代码大小: 0x{total_code_size:X}")
        print(f"✓ 实际数据大小: 0x{total_data_size:X}")
    
    def compare_sections(self, sections: List[Dict], pe_info: Dict):
        """对比段表"""
        if not self.ref_data:
            return
            
        print("=== 段表对比 ===")
        
        # 简化对比，主要看段数量和大小差异
        self.info.append(f"目标文件段数: {len(sections)}")
        
        total_virtual = sum(s['virtual_size'] for s in sections)
        total_raw = sum(s['raw_size'] for s in sections)
        
        self.info.append(f"总虚拟大小: 0x{total_virtual:X}")
        self.info.append(f"总原始大小: 0x{total_raw:X}")
        
        # 分析段名模式
        section_names = [s['name'] for s in sections]
        self.info.append(f"段名: {', '.join(section_names)}")
    
    def check_file_consistency(self, sections: List[Dict], opt_info: Dict):
        """检查文件整体一致性"""
        print("=== 检查文件一致性 ===")
        
        file_size = len(self.data)
        print(f"✓ 文件大小: {file_size} 字节 (0x{file_size:X})")
        
        # 1. 检查文件大小合理性
        if file_size % 512 != 0:
            self.warnings.append(f"文件大小不是512字节对齐: {file_size}")
        
        # 2. 计算理论最小文件大小
        min_size = opt_info['header_size']
        for section in sections:
            if section['raw_size'] > 0:
                section_end = section['raw_ptr'] + section['raw_size']
                min_size = max(min_size, section_end)
        
        print(f"✓ 理论最小大小: 0x{min_size:X}")
        
        if file_size < min_size:
            self.errors.append(f"文件被截断: {file_size} < {min_size}")
        elif file_size > min_size + 4096:  # 允许4KB的余量
            self.warnings.append(f"文件可能有多余数据: {file_size} > {min_size}")
        
        # 3. 检查空隙
        self.check_file_gaps(sections, opt_info)
        
        # 4. 验证关键区域
        self.verify_file_regions(sections, opt_info)
    
    def check_file_gaps(self, sections: List[Dict], opt_info: Dict):
        """检查文件中的空隙"""
        print("=== 检查文件空隙 ===")
        
        # 收集所有已使用的文件区域
        used_regions = [(0, opt_info['header_size'])]  # 头部区域
        
        for section in sections:
            if section['raw_size'] > 0:
                used_regions.append((section['raw_ptr'], section['raw_ptr'] + section['raw_size']))
        
        # 排序并合并重叠区域
        used_regions.sort()
        merged_regions = []
        for start, end in used_regions:
            if merged_regions and start <= merged_regions[-1][1]:
                merged_regions[-1] = (merged_regions[-1][0], max(merged_regions[-1][1], end))
            else:
                merged_regions.append((start, end))
        
        # 检查空隙
        file_size = len(self.data)
        gaps = []
        
        for i, (start, end) in enumerate(merged_regions):
            if i == 0 and start > 0:
                gaps.append((0, start))
            if i > 0:
                prev_end = merged_regions[i-1][1]
                if start > prev_end:
                    gaps.append((prev_end, start))
        
        if merged_regions and merged_regions[-1][1] < file_size:
            gaps.append((merged_regions[-1][1], file_size))
        
        if gaps:
            print("发现文件空隙:")
            for start, end in gaps:
                size = end - start
                print(f"  0x{start:X} - 0x{end:X} (大小: 0x{size:X})")
                if size > 1024:  # 大于1KB的空隙
                    self.warnings.append(f"大空隙: 0x{start:X}-0x{end:X} (0x{size:X}字节)")
        else:
            print("✓ 无文件空隙")
    
    def verify_file_regions(self, sections: List[Dict], opt_info: Dict):
        """验证关键文件区域"""
        print("=== 验证关键区域 ===")
        
        # 1. 检查入口点是否在代码段中
        entry_point = opt_info['entry_point']
        entry_found = False
        
        for section in sections:
            if (section['virtual_addr'] <= entry_point < 
                section['virtual_addr'] + section['virtual_size']):
                if section['characteristics'] & 0x20:  # CODE
                    entry_found = True
                    print(f"✓ 入口点在代码段{section['name']}中")
                else:
                    self.warnings.append(f"入口点在非代码段{section['name']}中")
                break
        
        if not entry_found:
            self.errors.append(f"入口点0x{entry_point:X}不在任何段中")
        
        # 2. 检查是否有可执行段
        has_executable = any(s['characteristics'] & 0x20000000 for s in sections)
        if not has_executable:
            self.warnings.append("没有可执行段")
        
        # 3. 检查零填充区域
        self.check_zero_regions(sections)
    
    def check_zero_regions(self, sections: List[Dict]):
        """检查零填充区域"""
        print("=== 检查零填充区域 ===")
        
        zero_regions = []
        chunk_size = 1024  # 1KB块检查
        
        for i in range(0, len(self.data), chunk_size):
            chunk = self.data[i:i+chunk_size]
            if len(chunk) == chunk_size and chunk == b'\x00' * chunk_size:
                zero_regions.append((i, i + chunk_size))
        
        # 合并连续的零区域
        if zero_regions:
            merged_zeros = []
            current_start = zero_regions[0][0]
            current_end = zero_regions[0][1]
            
            for start, end in zero_regions[1:]:
                if start == current_end:
                    current_end = end
                else:
                    merged_zeros.append((current_start, current_end))
                    current_start = start
                    current_end = end
            merged_zeros.append((current_start, current_end))
            
            print("发现零填充区域:")
            for start, end in merged_zeros:
                size = end - start
                print(f"  0x{start:X} - 0x{end:X} (大小: 0x{size:X})")
                
                # 检查是否在某个段的BSS区域
                in_bss = False
                for section in sections:
                    if (section['raw_size'] == 0 and section['virtual_size'] > 0 and
                        section['virtual_addr'] <= start < section['virtual_addr'] + section['virtual_size']):
                        in_bss = True
                        break
                
                if not in_bss and size > 4096:  # 大于4KB的非BSS零区域
                    self.warnings.append(f"大零填充区域: 0x{start:X}-0x{end:X} (0x{size:X}字节)")
        else:
            print("✓ 无大块零填充区域")
    
    def generate_summary_report(self, pe_info: Dict, opt_info: Dict, sections: List[Dict]):
        """生成总结报告"""
        print("\n" + "=" * 60)
        print("详细分析报告")
        print("=" * 60)
        
        # 基本信息
        print(f"文件: {self.file_path}")
        print(f"大小: {len(self.data)} 字节 (0x{len(self.data):X})")
        print(f"架构: 0x{pe_info['machine']:X}")
        print(f"段数: {pe_info['sections']}")
        print(f"入口点: 0x{opt_info['entry_point']:X}")
        print(f"镜像大小: 0x{opt_info['image_size']:X}")
        
        # 段信息汇总
        print(f"\n段信息汇总:")
        total_code = 0
        total_data = 0
        total_bss = 0
        
        for i, section in enumerate(sections):
            print(f"  {i}: {section['name']:8} VA=0x{section['virtual_addr']:06X} "
                  f"VS=0x{section['virtual_size']:06X} "
                  f"RS=0x{section['raw_size']:06X}")
            
            if section['characteristics'] & 0x20:  # CODE
                total_code += section['virtual_size']
            elif section['characteristics'] & 0x40:  # INITIALIZED_DATA
                total_data += section['virtual_size']
            elif section['raw_size'] == 0 and section['virtual_size'] > 0:
                total_bss += section['virtual_size']
        
        print(f"\n内存使用:")
        print(f"  代码段: 0x{total_code:X} 字节")
        print(f"  数据段: 0x{total_data:X} 字节")
        print(f"  BSS段:  0x{total_bss:X} 字节")
        print(f"  总计:   0x{total_code + total_data + total_bss:X} 字节")
        
        # 效率分析
        file_size = len(self.data)
        virtual_size = opt_info['image_size']
        
        print(f"\n效率分析:")
        print(f"  文件/内存比: {file_size/virtual_size:.2f}")
        if file_size > virtual_size:
            print(f"  ⚠️  文件大于内存镜像")
        
        # 对比结果
        if self.reference_path and self.ref_data:
            print(f"\n与参考文件对比:")
            print(f"  参考文件: {self.reference_path}")
            print(f"  参考大小: {len(self.ref_data)} 字节")
            ratio = len(self.data) / len(self.ref_data)
            print(f"  大小比例: {ratio:.2f}")
            
            if ratio < 0.1:
                print(f"  📊 目标文件比参考文件小很多")
            elif ratio > 2.0:
                print(f"  📊 目标文件比参考文件大很多")
    
    def run_check(self) -> bool:
        """运行完整检查"""
        print(f"检查EFI镜像: {self.file_path}")
        if self.reference_path:
            print(f"参考文件: {self.reference_path}")
        print("=" * 80)
        
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
        
        self.check_file_consistency(sections, opt_info)
        
        # 生成详细报告
        self.generate_summary_report(pe_info, opt_info, sections)
        
        # 报告结果
        print("\n" + "=" * 80)
        print("检查结果:")
        
        if self.errors:
            print(f"❌ 发现 {len(self.errors)} 个错误:")
            for i, error in enumerate(self.errors, 1):
                print(f"  {i}. {error}")
        
        if self.warnings:
            print(f"⚠️  发现 {len(self.warnings)} 个警告:")
            for i, warning in enumerate(self.warnings, 1):
                print(f"  {i}. {warning}")
        
        if self.info:
            print(f"ℹ️  信息 ({len(self.info)} 项):")
            for i, info in enumerate(self.info, 1):
                print(f"  {i}. {info}")
        
        if not self.errors and not self.warnings:
            print("✅ 所有检查通过！")
        elif not self.errors:
            print("✅ 结构验证通过（有警告）")
        else:
            print("❌ 发现严重错误")
        
        success = len(self.errors) == 0
        return success

def main():
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print("用法: python3 check_efi.py <efi_file> [reference_efi_file]")
        print("示例:")
        print("  python3 check_efi.py target/kernel.efi")
        print("  python3 check_efi.py target/kernel.efi tool/vmlinux.efi")
        sys.exit(1)
        
    efi_file = sys.argv[1]
    reference_file = sys.argv[2] if len(sys.argv) == 3 else None
    
    # 如果没有指定参考文件，但存在默认的参考文件，自动使用
    if not reference_file:
        default_ref = "tool/vmlinux.efi"
        if os.path.exists(default_ref) and os.path.abspath(efi_file) != os.path.abspath(default_ref):
            reference_file = default_ref
            print(f"自动使用参考文件: {reference_file}")
    
    checker = EFIImageChecker(efi_file, reference_file)
    
    success = checker.run_check()
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()