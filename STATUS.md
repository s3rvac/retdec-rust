# Status

A summary of the supported parts of the [retdec.com's API](https://retdec.com/api/docs/index.html).

## Decompiler

The decompilation service.

* [Starting a new decompilation](https://retdec.com/api/docs/decompiler.html#starting-a-new-decompilation>) ✔
* [Decompilation modes](https://retdec.com/api/docs/decompiler.html#decompilation-modes>) (partial)
  * `bin` ✔
  * `c` ✗
  * `raw` ✗
* [Input files](https://retdec.com/api/docs/decompiler.html#input-files>) (partial)
  * `input` ✔
  * `pdb` ✗
* [Decompilation parameters](https://retdec.com/api/docs/decompiler.html#decompilation-parameters>) ✗
  * [Mode-independent parameters](https://retdec.com/api/docs/decompiler.html#mode-independent-parameters>) ✗
    * `target_language` ✗
    * `graph_format` ✗
    * `decomp_var_names` ✗
    * `decomp_optimizations` ✗
    * `decomp_unreach_funcs` ✗
    * `decomp_emit_addresses` ✗
    * `generate_cg` ✗
    * `generate_cfgs` ✗
    * `generate_archive` ✗
  * [Parameters for the bin mode](https://retdec.com/api/docs/decompiler.html#parameters-only-for-the-bin-mode>) ✗
    * `architecture` ✗
    * `endian` ✗
    * `sel_decomp_funcs` ✗
    * `sel_decomp_ranges` ✗
    * `sel_decomp_decoding` ✗
    * `ar_index` ✗
    * `ar_name` ✗
  * [Parameters for the raw mode](https://retdec.com/api/docs/decompiler.html#parameters-only-for-the-raw-mode>) ✗
    * `architecture` ✗
    * `endian` ✗
    * `raw_entry_point` ✗
    * `raw_section_vma` ✗
  * [Parameters for the c mode](https://retdec.com/api/docs/decompiler.html#parameters-only-for-the-c-mode>) ✗
    * `architecture` ✗
    * `file_format` ✗
    * `comp_compiler` ✗
    * `comp_optimizations` ✗
    * `comp_debug` ✗
    * `comp_strip` ✗
* [Checking status](https://retdec.com/api/docs/decompiler.html#checking-status>) (partial)
  * general (`running`, `finished`, etc.) (partial)
  * `completion` ✗
  * `phases` ✗
    * `part` ✗
    * `name` ✗
    * `description` ✗
    * `completion` ✗
    * `warnings` ✗
  * `cg` ✗
  * `cfgs` ✗
  * `archive` ✗
* [Obtaining outputs](https://retdec.com/api/docs/decompiler.html#obtaining-outputs>) (partial)
  * `hll` ✔
  * `dsm` ✗
  * `cg` ✗
  * `cfgs` ✗
  * `archive` ✗
  * `binary` ✗
* [Error reporting](https://retdec.com/api/docs/decompiler.html#error-reporting>) (partial)

## Fileinfo

The file-analyzing service.

* [Starting a new analysis](https://retdec.com/api/docs/fileinfo.html#starting-a-new-analysis>) ✔
* [Optional parameters](https://retdec.com/api/docs/fileinfo.html#optional-parameters>) ✔
  * `output_format` ✔
  * `verbose` ✔
* [Checking status](https://retdec.com/api/docs/fileinfo.html#checking-status>) (partial)
  * general (`running`, `finished`, etc.) (partial)
* [Obtaining output](https://retdec.com/api/docs/fileinfo.html#obtaining-output>) ✔
* [Error reporting](https://retdec.com/api/docs/fileinfo.html#error-reporting>) (partial)

## Test

The testing service.

* [Authentication](https://retdec.com/api/docs/test.html#authentication>) ✗
* [Parameter passing](https://retdec.com/api/docs/test.html#parameter-passing>) ✗
