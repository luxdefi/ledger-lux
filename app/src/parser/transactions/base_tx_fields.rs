/*******************************************************************************
*   (c) 2021 Zondax GmbH
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/
use core::ops::Deref;

use core::{mem::MaybeUninit, ptr::addr_of_mut};
use nom::{bytes::complete::take, number::complete::be_u32};

use crate::parser::{
    DisplayableItem, FromBytes, ObjectList, Output, OutputIdx, ParserError, TransferableInput,
    TransferableOutput,
};

const MAX_MEMO_LEN: usize = 256;

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct BaseTxFields<'b, O>
where
    O: FromBytes<'b> + DisplayableItem + Deref<Target = Output<'b>> + 'b,
{
    // lazy parsing of inputs/outpus
    pub outputs: ObjectList<'b, TransferableOutput<'b, O>>,
    // a bit-wise idx that tells what outputs could be displayed
    // in the ui stage.
    // this is set during the parsing stage.
    renderable_out: OutputIdx,
    // inputs can be generic as well.
    // but so far, there is only one input
    // across all chains and their transactions.
    // if this change in the future, we can make
    // it generic over any input-type, in the maintenance cycle
    pub inputs: ObjectList<'b, TransferableInput<'b>>,
    pub memo: &'b [u8],
}

impl<'b, O> BaseTxFields<'b, O>
where
    O: FromBytes<'b> + DisplayableItem + Deref<Target = Output<'b>> + 'b,
{
    pub fn disable_output_if(&mut self, address: &[u8]) {
        let num_outs = self.outputs.iter().count();
        // skip filtering out outputs if there is only one
        if num_outs <= 1 {
            return;
        }
        self.outputs.iter().enumerate().for_each(|(idx, o)| {
            if o.num_addresses() == 1 && o.contain_address(address) {
                self.renderable_out ^= 1 << idx;
            }
        });
    }

    pub fn sum_inputs_amount(&self) -> Result<u64, ParserError> {
        self.inputs
            .iter()
            .map(|input| input.amount().ok_or(ParserError::UnexpectedError))
            .try_fold(0u64, |acc, x| {
                acc.checked_add(x?).ok_or(ParserError::OperationOverflows)
            })
    }

    pub fn sum_outputs_amount(&'b self) -> Result<u64, ParserError> {
        self.outputs
            .iter()
            .map(|output| (*output).amount().ok_or(ParserError::UnexpectedError))
            .try_fold(0u64, |acc, x| {
                acc.checked_add(x?).ok_or(ParserError::OperationOverflows)
            })
    }

    pub fn outputs(&'b self) -> &ObjectList<TransferableOutput<O>> {
        &self.outputs
    }

    pub fn inputs(&self) -> &ObjectList<TransferableInput> {
        &self.inputs
    }

    pub fn base_outputs_num_items(&'b self) -> usize {
        let mut items = 0;
        let mut idx = 0;
        self.outputs.iterate_with(|o| {
            let render = self.renderable_out & (1 << idx);
            if render > 0 {
                items += o.num_items();
            }
            idx += 1;
        });
        items
    }

    // Gets the obj that contain the item_n, along with the index
    // of the item. Returns an error otherwise
    pub fn base_output_with_item(
        &'b self,
        item_n: u8,
    ) -> Result<(TransferableOutput<O>, u8), ParserError> {
        let mut count = 0usize;
        let mut obj_item_n = 0;
        // index to check for renderable outputs.
        // we can omit this and be "fancy" with iterators but
        // they consume a lot of stack.
        // causing stack overflows in nanos
        let mut idx = 0;
        // gets the output that contains item_n
        // and its corresponding index
        let filter = |o: &TransferableOutput<'b, O>| -> bool {
            // filter out
            let render = self.renderable_out & (1 << idx) > 0;
            idx += 1;

            if !render {
                return false;
            }

            let n = o.num_items();
            for index in 0..n {
                obj_item_n = index;
                if count == item_n as usize {
                    return true;
                }
                count += 1;
            }
            false
        };

        let obj = self
            .outputs()
            .get_obj_if(filter)
            .ok_or(ParserError::DisplayIdxOutOfRange)?;

        Ok((obj, obj_item_n as u8))
    }
}

impl<'b, O> FromBytes<'b> for BaseTxFields<'b, O>
where
    O: FromBytes<'b> + DisplayableItem + Deref<Target = Output<'b>> + 'b,
{
    #[inline(never)]
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        crate::sys::zemu_log_stack("BaseTxFields::from_bytes_into\x00");

        let out = out.as_mut_ptr();
        // check for the number of outputs before parsing then as now
        // it has to be checked for the outputIdx capacity which is used
        // to tell if an output should be rendered or not.
        let (_, num_outputs) = be_u32(input)?;
        if num_outputs > OutputIdx::BITS {
            return Err(ParserError::TooManyOutputs.into());
        }
        // get outputs
        let outputs = unsafe { &mut *addr_of_mut!((*out).outputs).cast() };
        let rem = ObjectList::<TransferableOutput<O>>::new_into(input, outputs)?;

        // inputs
        let inputs = unsafe { &mut *addr_of_mut!((*out).inputs).cast() };
        let rem = ObjectList::<TransferableInput>::new_into(rem, inputs)?;

        // memo
        let (rem, memo_len) = be_u32(rem)?;

        if memo_len as usize > MAX_MEMO_LEN {
            return Err(ParserError::ValueOutOfRange.into());
        }

        let (rem, memo) = take(memo_len as usize)(rem)?;

        //good ptr and no uninit reads
        unsafe {
            addr_of_mut!((*out).memo).write(memo);
            // by default all outputs are renderable
            addr_of_mut!((*out).renderable_out).write(OutputIdx::MAX);
        }

        Ok(rem)
    }
}
