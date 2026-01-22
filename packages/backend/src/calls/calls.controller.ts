import { 
  Controller, 
  Get, 
  Post, 
  Body, 
  Param, 
  Query, 
  HttpCode, 
  HttpStatus,
  ValidationPipe,
  UsePipes
} from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse, ApiParam, ApiQuery } from '@nestjs/swagger';
import { CallsService, CallStatus, CallFilter } from './calls.service';
import { CreateCallDto } from './dto/create-call.dto';

@ApiTags('calls')
@Controller('calls')
export class CallsController {
  constructor(private readonly callsService: CallsService) {}

  @Get()
  @ApiOperation({ summary: 'List calls with filters' })
  @ApiQuery({ name: 'status', required: false, description: 'Filter by call status', enum: CallStatus })
  @ApiQuery({ name: 'creator', required: false, description: 'Filter by creator address' })
  @ApiQuery({ name: 'limit', required: false, description: 'Number of items per page', type: Number })
  @ApiQuery({ name: 'offset', required: false, description: 'Offset for pagination', type: Number })
  @ApiResponse({ status: 200, description: 'Successfully retrieved calls.' })
  @ApiResponse({ status: 400, description: 'Invalid query parameters.' })
  async findAll(
    @Query('status') status?: CallStatus,
    @Query('creator') creator?: string,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const filters: CallFilter = {
      status,
      creatorAddress: creator,
      limit: limit ? parseInt(limit.toString(), 10) : undefined,
      offset: offset ? parseInt(offset.toString(), 10) : undefined,
    };

    return this.callsService.findAll(filters);
  }

  @Get(':id')
  @ApiOperation({ summary: 'Get call details with participants' })
  @ApiParam({ name: 'id', description: 'Call ID', type: Number })
  @ApiResponse({ status: 200, description: 'Successfully retrieved call.' })
  @ApiResponse({ status: 404, description: 'Call not found.' })
  async findOne(@Param('id') id: string) {
    const callId = parseInt(id, 10);
    if (isNaN(callId)) {
      throw new Error('Invalid call ID');
    }
    
    return this.callsService.findOne(callId);
  }

  @Post('draft')
  @HttpCode(HttpStatus.CREATED)
  @ApiOperation({ summary: 'Create call draft and pin to IPFS' })
  @ApiResponse({ status: 201, description: 'Call draft created successfully.' })
  @ApiResponse({ status: 400, description: 'Invalid input data.' })
  @UsePipes(new ValidationPipe({ whitelist: true, forbidNonWhitelisted: true }))
  async createDraft(@Body() createCallDto: CreateCallDto) {
    return this.callsService.createDraft(createCallDto);
  }

  @Get('user/:address')
  @ApiOperation({ summary: 'Get calls by creator' })
  @ApiParam({ name: 'address', description: 'Creator address' })
  @ApiQuery({ name: 'limit', required: false, description: 'Number of items per page', type: Number })
  @ApiQuery({ name: 'offset', required: false, description: 'Offset for pagination', type: Number })
  @ApiResponse({ status: 200, description: 'Successfully retrieved user calls.' })
  async findByUser(
    @Param('address') address: string,
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const filters: CallFilter = {
      limit: limit ? parseInt(limit.toString(), 10) : undefined,
      offset: offset ? parseInt(offset.toString(), 10) : undefined,
    };

    return this.callsService.findByUser(address, filters);
  }

  @Get('feed')
  @ApiOperation({ summary: 'Get trending/recent calls for feed' })
  @ApiQuery({ name: 'limit', required: false, description: 'Number of of items per page', type: Number })
  @ApiQuery({ name: 'offset', required: false, description: 'Offset for pagination', type: Number })
  @ApiResponse({ status: 200, description: 'Successfully retrieved feed calls.' })
  async getFeed(
    @Query('limit') limit?: number,
    @Query('offset') offset?: number,
  ) {
    const filters: CallFilter = {
      limit: limit ? parseInt(limit.toString(), 10) : undefined,
      offset: offset ? parseInt(offset.toString(), 10) : undefined,
    };

    return this.callsService.getFeed(filters);
  }
}